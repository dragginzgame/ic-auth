#!/usr/bin/env perl
# Auth's receipt envelope and retained-byte custody; Shared alone admits a run.
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../../vendor/toko-tooling/scripts/ci";
use ValidationPlan qw(json bytes digest document relative_path keys_are);
use JSON::PP;
use Digest::SHA qw(sha256_hex);
use Cwd qw(getcwd);
my ($operation, $path) = @ARGV;
@ARGV == 2 && $operation =~ /\A(?:collect|check|check-retained|check-record)\z/
    or die "usage: validation-record.pl collect|check|check-retained DIRECTORY | check-record RECEIPT\n";
my $root = getcwd();
my $decoder = JSON::PP->new->utf8;
sub directory_ok {
    relative_path($_[0]);
    $_[0] =~ m{\A(?:\.git/release-state|target)/[A-Za-z0-9_./-]+\z}
        or die "foreign Auth gate directory\n";
}
sub shared_proof {
    my ($directory, $context, $run) = @_;
    open my $in, '-|', $^X, 'vendor/toko-tooling/scripts/ci/validation-evidence.pl', 'verify', $root,
        "$directory/plan.json", $context->{source}, $context->{group}, $run,
        "$directory/context.json" or die "start Shared evidence admission: $!\n";
    binmode $in;
    my $raw = '';
    while (1) {
        my $n = read $in, my $chunk, 65536;
        defined($n) or die "read Shared proof: $!\n";
        last unless $n;
        $raw .= $chunk;
        length($raw) <= 1048576 or die "oversized Shared proof\n";
    }
    close $in or die "Shared run evidence refused\n";
    return $decoder->decode($raw);
}
sub admit_record {
    my ($record) = @_;
    keys_are($record, qw(schema directory source group scope context plan proof proof_sha256));
    $record->{schema} == 2 or die "historical Auth evidence needs its original adapter\n";
    directory_ok($record->{directory});
    my ($plan, $context, $proof) = @{$record}{qw(plan context proof)};
    $record->{source} eq $context->{source} && $record->{source} eq $plan->{source} &&
        $record->{group} eq $context->{group} && $record->{group} eq $plan->{group} &&
        $record->{scope} eq $plan->{scope} or die "Auth record identity differs\n";
    # The immutable receipt binds Shared's opaque result to Auth's admitted source
    # and selection. Do not implement Shared's stage/event/schema parser here.
    sha256_hex(json($proof)) eq $record->{proof_sha256} &&
        $proof->{kind} eq 'group-validation-evidence' && $proof->{schema} == 1 &&
        $proof->{source} eq $record->{source} && $proof->{group} eq $record->{group} &&
        $proof->{scope} eq $record->{scope} &&
        $proof->{context_sha256} eq sha256_hex(json($context)) &&
        $proof->{plan_sha256} eq sha256_hex(json($plan)) &&
        $proof->{policy_sha256} eq $context->{inputs}{$plan->{policy}}
        or die "Auth receipt lost its original Shared proof binding\n";
    relative_path($proof->{run});
    $proof->{run} =~ m{\A\Q$record->{directory}\E/logs/validation\.[A-Za-z0-9]+\z}
        or die "Shared proof belongs to another Auth gate\n";
    return $record;
}
if ($operation eq 'check-record') {
    my $receipt = $decoder->decode(bytes($root, $path));
    $receipt->{schema} == 4 && $receipt->{gate} eq 'release-plan'
        or die "historical gate receipt needs its original adapter\n";
    print json(admit_record($receipt->{validation}));
    exit 0;
}
directory_ok($path);
my $context = document($root, "$path/context.json");
my $plan = document($root, "$path/plan.json");
if ($operation eq 'collect') {
    my @runs = glob "$path/logs/validation.*";
    @runs == 1 or die "missing or ambiguous Auth run\n";
    my $proof = shared_proof($path, $context, $runs[0]);
    print json(admit_record({schema=>2, directory=>$path, source=>$plan->{source},
        group=>$plan->{group}, scope=>$plan->{scope}, context=>$context, plan=>$plan,
        proof=>$proof, proof_sha256=>sha256_hex(json($proof))}));
    exit 0;
}
my $record = admit_record(document($root, "$path/evidence.json"));
json($record->{context}) eq json($context) && json($record->{plan}) eq json($plan)
    or die "retained Auth selection changed\n";
if ($operation eq 'check') {
    json(shared_proof($path, $context, $record->{proof}{run})) eq json($record->{proof})
        or die "Shared proof changed\n";
} else {
    # Prepared metadata legitimately differs. The metadata owner first admits
    # the original source/context, then checks retained bytes without projecting
    # new versions or reinterpreting the original execution records.
    my $files = $record->{proof}{files};
    ref($files) eq 'HASH' && keys(%$files) or die "missing retained proof files\n";
    for my $file (keys %$files) {
        digest($root, "$record->{proof}{run}/$file") eq $files->{$file}
            or die "retained Shared evidence changed: $file\n";
    }
}
print json($record);
