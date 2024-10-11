pragma circom 2.1.6;

include "blake3_compression.circom";
include "risc0.circom";
include "../../circomlib/circuits/bitify.circom";

template Blake3 () {
    signal input inp[16]; // 16 32-bit words
    signal output out[8]; // 8 32-bit words

    component iv = IV();
    component blake3 = Blake3Compression();
    for (var i = 0; i < 8; i++) {
        blake3.h[i] <== iv.out[i];
    }
    for (var i = 0; i < 16; i++) {
        blake3.m[i] <== inp[i];
    }
    blake3.t[0] <== 0;
    blake3.t[1] <== 0;
    blake3.b <== 64;
    blake3.d <== 11;


    for (var i = 0; i < 8; i++) {
        out[i] <== blake3.out[i];
    }
}

template Blake3_with_scalar_output () {
    signal input inp[16]; // 16 32-bit words
    signal output out;
    signal outbits[256];

    component blake3 = Blake3();
    for (var i = 0; i < 16; i++) {
        blake3.inp[i] <== inp[i];
    }

    component to_bits[8];
    for (var i = 0; i < 8; i++) {
        to_bits[i] = to_bits_exact(32);
        to_bits[i].in <== blake3.out[i];
        for (var j = 0; j < 32; j++) {
            outbits[i*32 + j] <== to_bits[i].out[j];
        }
    }

    component to_num = Bits2Num(248); // We delete the last 8 bits

    for (var i = 0; i < 248; i++) {
        var index = (248 - i - 1) - (248 - i - 1)%8 + i%8;
        to_num.in[index] <== outbits[i];
    }
    out <== to_num.out;
}

// component main = Blake3_with_scalar_output();