pragma circom 2.0.4;

include "stark_verify.circom";
include "journal.circom";

template VerifyForGuest() {
    signal input journal[32];
    signal input iop[25749];
    component stark_verifier = Verify();
    component claim = Journal();

    for (var i = 0; i < 25749; i++) {
        stark_verifier.iop[i] <== iop[i];
    }

    for (var i = 0; i < 32; i++) {
        claim.in[i] <== journal[i];
    }

    stark_verifier.out[2] === claim.out[0];
    stark_verifier.out[3] === claim.out[1];
}

component main { public [ journal ] } = VerifyForGuest();
