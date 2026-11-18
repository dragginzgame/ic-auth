use strict;
use warnings;
use FindBin;
use lib "$FindBin::Bin/../../vendor/toko-tooling/scripts/ci";
use SetupDiagnostics qw(run_setup);

# The workflow tees only the bounded failure excerpt into its retained log.
exit run_setup(sub {
    my ($status, $excerpt) = @_;
    print STDERR "Setup failed (exit $status)\n", $excerpt;
}, @ARGV);
