#!/usr/bin/env perl
use strict;
use warnings;

use Digest::FNV 'fnv32a';

while(<>) {
    # Skip comments
    next if /^#/;
    # Get the key
    my ($key) = /"(.*)"/;
    # Get the hash as bytes
    my $hashed_key;
    # Convert hash to hex and make sure backslashes were escaped
    # Hash of every char | 0x20
    my $hash_input = '';
    for my $key_char (split //, $key =~ s/\\\\/\\/gr) {
        $hash_input .= chr(ord($key_char) | 0x20);
    }
    
    for my $char (split //, pack("V", fnv32a($hash_input))) {
        $hashed_key .= sprintf("\\x%02X", ord($char));
    }
    # Output in desired format
    print "b\"$hashed_key\" => \"$key\",\n"
}
