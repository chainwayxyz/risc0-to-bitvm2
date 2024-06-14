pragma circom 2.0.4;

include "stark_verify.circom";

template VerifyForGuest() {
    // signal input journal[32];
    signal input iop[25749];
    signal output c0;
    component stark_verifier = Verify();

    for (var i = 0; i < 25749; i++) {
        stark_verifier.iop[i] <== iop[i];
    }
    c0 <== stark_verifier.out[2];
}

component main = VerifyForGuest();
