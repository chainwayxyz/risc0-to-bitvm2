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

    log("out_0");
    log(stark_verifier.out[0]);
    log("out_1");
    log(stark_verifier.out[1]);
    log("out_2");
    log(stark_verifier.out[2]);
    log("out_3");
    log(stark_verifier.out[3]);
    log("codeRoot");
    log(stark_verifier.codeRoot);
    log("claim_out_0");
    log(claim.out[0]);
    log("claim_out_1");
    log(claim.out[1]);
    stark_verifier.out[0] === 19350802088444617183621339156085479077;
    stark_verifier.out[1] === 61803236023146647725736150410140474743;
    stark_verifier.out[2] === claim.out[0];
    stark_verifier.out[3] === claim.out[1];

    stark_verifier.codeRoot === 6655704183316983190945468237220041514376883004657559498672647785620383118673;
}

component main { public [ journal ] } = VerifyForGuest();
