#!/usr/bin/env perl
use strict;
use warnings;
use FindBin qw($Bin);

# Deliberately narrow byte-preserving edits to this root's maintained scalar and
# inline dependency declarations. Cargo/yq validate the surrounding TOML; fail
# if a future layout needs a different transformation rather than guessing.
@ARGV == 3 or die "usage: rewrite-manifest.pl MANIFEST PREVIOUS CANDIDATE\n";
my ($path, $previous, $candidate) = @ARGV;
for ($previous, $candidate) {
    /\A(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\z/
        or die "invalid stable version\n";
}
open my $rendered, '-|', $^X, "$Bin/../ci/render-cargo-workspace-version.pl",
    $path, $previous, $candidate or die "start workspace renderer: $!\n";
binmode $rendered or die "workspace renderer pipe: $!\n";
my $text = do { local $/; <$rendered> };
close $rendered or die "workspace renderer failed\n";
defined $text or die "workspace renderer returned no manifest\n";
my $dependency = ($text =~ s/^(ic-auth-protocol-types = \{[^\n]*?\bversion = ")\Q$previous\E(")/$1$candidate$2/mg);
$dependency == 1 or die "unexpected root dependency declaration layout\n";
print $text or die "manifest output: $!\n";
