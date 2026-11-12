#!/usr/bin/env perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../ci";
use ValidationPlan qw(bytes document policy);
use Cwd qw(getcwd abs_path);
use Encode qw(decode FB_CROAK);
use JSON::PP;
use Digest::SHA qw(sha256_hex);
$ENV{RUSTUP_AUTO_INSTALL} = 0;
my ($source, $group, $previous, $directory, @options) = @ARGV;
defined($directory) or die "usage: validation-plan.pl SOURCE GROUP PREVIOUS DIRECTORY [--baseline REF] [--workspace]\n";
my $json = JSON::PP->new->canonical->utf8;
my ($source_root, $check, $workspace, $record_path) = ('', 0, 0, '');
my $baseline_ref = "refs/tags/$group-v$previous";
while (@options) {
    my $option = shift @options;
    if ($option eq '--workspace') { $workspace = 1; }
    elsif ($option eq '--check') { $check = 1; }
    elsif ($option eq '--baseline' && @options) { $baseline_ref = shift @options; }
    elsif ($option eq '--source-root' && @options) { $source_root = shift @options; }
    elsif ($option eq '--record' && @options) { $record_path = shift @options; }
    else { die "unknown plan option\n"; }
}
my $root = getcwd();
$source_root = abs_path($source_root) // die "missing original source root\n" if $source_root ne '';
my $selected_root = $source_root eq '' ? $root : $source_root;
sub command {
    open my $in, '-|', @_ or die "execute $_[0]: $!\n";
    binmode $in;
    my $raw = '';
    while (1) {
        my $n = read $in, my $chunk, 65536;
        defined($n) or die "read observation: $!\n";
        last unless $n;
        $raw .= $chunk;
        length($raw) <= 4194304 or die "oversized source observation\n";
    }
    close $in or die "command failed: @_\n";
    return $raw;
}
$source =~ /\A[0-9a-f]{40}(?:[0-9a-f]{24})?\z/ &&
    $previous =~ /\A\d+\.\d+\.\d+\z/ or die "invalid source/version\n";
command('git', 'rev-parse', '--verify', "$source\^{commit}") eq "$source\n" or die "invalid source commit\n";
$check || command('git', 'rev-parse', 'HEAD') eq "$source\n" or die "selection source moved\n";
my $policy_path = 'ci/release-validation.json';
my $policy = policy($selected_root, $policy_path);
my $coverage = document($selected_root, 'ci/release-validation-coverage.json');
my $groups = $json->decode(bytes($selected_root, 'ci/cargo-release-groups.json'));
my @groups = sort map { $_->{name} } @{$groups->{groups}};
join(',', @groups) eq join(',', sort @{$policy->{groups}}) &&
    join(',', @groups) eq join(',', sort keys %$coverage) or die "release group roster mismatch\n";
grep($_ eq $group, @groups) or die "unknown group\n";
my %stages;
for (@{$policy->{stages}}) {
    /\Arelease-stage-([a-z0-9-]+)\z/ or die "Auth stage lacks its release wrapper\n";
    $stages{$1} = 1;
}
for my $owner (@groups) {
    for my $target (keys %{$coverage->{$owner}}) {
        $stages{$target} && $coverage->{$owner}{$target} =~ /\S/ or die "unknown/missing exclusion reason\n";
    }
}
my ($owner) = grep { $_->{name} eq $group } @{$groups->{groups}};
my ($baseline, $baseline_tag, @changed) = ('', '');
if ($baseline_ref eq $source) {
    # Explicit zero-delta qualification only; release callers never supply this.
    $baseline = $source;
} else {
    my $observed = $json->decode(command($^X, 'scripts/ci/validation-evidence.pl',
        'baseline', $root, $source, $baseline_ref));
    if ($observed->{available}) {
        $baseline = $observed->{baseline};
        $baseline_tag = $observed->{object};
        @changed = @{$observed->{changes}};
        system('bash', 'scripts/ci/check-release-tag.sh', $baseline, $previous, $group) == 0
            or die "baseline must be the exact annotated group release\n";
    } elsif (exists $observed->{object}) {
        die "untrusted Auth baseline: $observed->{reason}\n";
    }
}
my $narrow = !$workspace && $baseline ne '';
my $selector_unchanged = $narrow;
my @broadening;
for my $path (@changed) {
    next if $path eq $owner->{changelog} || $path eq 'docs/status/current.md';
    my $owned = $group eq 'protocol'
        ? $path =~ m{\Acrates/ic-auth-(?:protocol-types|wire)/(?:src|tests)/[^\r\n]*\.rs\z}
        : $path =~ m{\A(?:crates/ic-auth|qualification/ic-auth-(?:qualification(?:-[^/]+)?|backend-conformance|feature-unification))/(?:src|tests)/[^\r\n]*\.rs\z};
    $narrow = 0 unless $owned;
    # This fixture executes shell/Perl selection and receipt admission with
    # substituted leaf bodies, never product code or a lock-selected Rust tool.
    # Lock changes still require every product/tool/archive obligation. Only
    # this synthetic regression is unaffected by these reviewed data inputs.
    my $selector_data = $owned || $path =~ m{\A(?:Cargo\.lock|qualification/Cargo\.lock|README\.md|docs/(?:integration|protocol)/[^/]+\.md|packages/client/src/generated/[^/]+\.ts)\z};
    unless ($selector_data) {
        $selector_unchanged = 0;
        push @broadening, $path;
    }
}
my @files = map { decode('UTF-8', $_, FB_CROAK) } split /\0/, command('git', 'ls-tree', '-r', '--name-only', '-z', $source);
my %inputs;
for my $path (@files) {
    $path !~ /[\x00\t\r\n]/ or die "unsafe tracked path\n";
    $inputs{$path} = sha256_hex(bytes($selected_root, $path));
}
my %environment;
my @environment_keys = qw(RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER CARGO_TARGET_DIR CC CFLAGS POCKET_IC_BIN IC_AUTH_BROWSER_EXECUTABLE);
for my $key (@environment_keys) {
    $environment{$key} = $ENV{$key} if exists $ENV{$key};
}
my $record;
if ($record_path ne '') {
    $check && $source_root ne '' or die "committed record needs exported source and check mode\n";
    my $receipt = $json->decode(bytes($root, $record_path));
    $receipt->{schema} == 4 && $receipt->{gate} eq 'release-plan' or die "not a committed gate receipt\n";
    $record = $receipt->{validation};
    my $origin = $record->{context};
    ref($origin->{environment}) eq 'HASH' or die "invalid origin environment\n";
    my %allowed = map { $_ => 1 } @environment_keys;
    for my $key (keys %{$origin->{environment}}) {
        $allowed{$key} && defined($origin->{environment}{$key}) && !ref($origin->{environment}{$key})
            or die "invalid origin environment value\n";
    }
    %environment = %{$origin->{environment}};
}
my $context = {schema=>1, source=>$source, group=>$group, baseline=>$baseline, baseline_ref=>$baseline_ref, baseline_tag=>$baseline_tag,
    changed=>\@changed, inputs=>\%inputs, environment=>\%environment,
    rustc=>command('rustc','-vV'), cargo=>command('cargo','-V'), host=>command('uname','-srm')};
if ($record) {
    for my $key (qw(rustc cargo host)) {
        defined($record->{context}{$key}) && !ref($record->{context}{$key}) && length($record->{context}{$key})
            or die "missing original execution context\n";
        $context->{$key} = $record->{context}{$key};
    }
}
my $context_path = "$directory/context.json";
my $plan = {schema=>1,source=>$source,group=>$group,scope=>($narrow || $selector_unchanged) ? 'group' : 'workspace',
    policy=>$policy_path,inputs=>[$context_path, 'Makefile', 'Cargo.toml', 'Cargo.lock', $owner->{changelog},
        'ci/cargo-release-groups.json', 'ci/release-validation-coverage.json',
        'scripts/release/validation-plan.pl', 'scripts/release/validation.sh', 'scripts/release/validation-stage.sh',
        'scripts/release/validation-record.pl', 'scripts/ci/validation-evidence.pl', 'scripts/ci/ValidationPlan.pm',
        'scripts/ci/validation-input-checkpoint.pl', 'scripts/ci/release-receipts.sh'], stages=>[
    map {
        my $stage = $_; (my $target = $stage) =~ s/^release-stage-//;
        my $excluded = exists $coverage->{$group}{$target} &&
            ($narrow || ($selector_unchanged && $target eq 'test-release-validation'));
        {target=>$stage, required=>$excluded ? JSON::PP::false : JSON::PP::true,
            reason=>$excluded ? $coverage->{$group}{$target} : 'Required common or affected Auth consumer obligation'}
    } @{$policy->{stages}}]};
unless ($check) {
    my $why = $workspace ? 'explicit complete-roster qualification'
        : $baseline eq '' ? 'no admitted previous release baseline'
        : @broadening ? 'changed infrastructure or unknown inputs: '.join(', ', @broadening)
        : $narrow ? 'owner source only; unchanged infrastructure fixtures excluded'
        : 'product/data changes; all obligations except unchanged selector regression required';
    warn "Auth validation selection: $plan->{scope}: $why\n";
}
for my $entry (['context.json',$context], ['plan.json',$plan]) {
    my $path = "$directory/$entry->[0]";
    if ($check) {
        my $observed = $record
            ? $json->encode($record->{$entry->[0] eq 'context.json' ? 'context' : 'plan'})."\n"
            : bytes($root, $path);
        $observed eq $json->encode($entry->[1])."\n" or die "saved group selection changed: $path\n";
        next;
    }
    !-e $path && !-l $path or die "selection output exists\n";
    open my $out, '>:raw', $path or die "write $path: $!\n";
    print {$out} $json->encode($entry->[1]), "\n" or die "write $path\n";
    close $out or die "close $path\n";
}
