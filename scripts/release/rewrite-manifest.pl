#!/usr/bin/env perl
use strict;
use warnings;

# Deliberately narrow byte-preserving edits to this root's maintained scalar and
# inline dependency declarations. Cargo/yq validate the surrounding TOML; fail
# if a future layout needs a different transformation rather than guessing.
@ARGV == 3 or die "usage: rewrite-manifest.pl MANIFEST PREVIOUS CANDIDATE\n";
my ($path, $previous, $candidate) = @ARGV;
for ($previous, $candidate) {
    /\A(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\z/
        or die "invalid stable version\n";
}
open my $file, '<', $path or die "$path: $!\n";
my $text = do { local $/; <$file> };
my $package = ($text =~ s/^version = "\Q$previous\E"$/version = "$candidate"/mg);
my $dependency = ($text =~ s/^(ic-auth-protocol-types = \{[^\n]*?\bversion = ")\Q$previous\E(")/$1$candidate$2/mg);
$package == 1 && $dependency == 1 or die "unexpected root version/dependency declaration layout\n";
print $text or die "manifest output: $!\n";
