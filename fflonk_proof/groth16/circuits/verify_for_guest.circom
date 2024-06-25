pragma circom 2.0.4;

include "stark_verify.circom";
include "journal.circom";

template VerifyForGuest(n) {
    signal input journal[(n \ 254)+1];
    component input_n2b[(n \ 254)];
    component input_last_n2b = Num2Bits((n % 254));

    for (var i = 0; i < (n \ 254); i++) {
        input_n2b[i] = Num2Bits(254);
        input_n2b[i].in <== journal[i];
    }
    input_last_n2b.in <== journal[(n \ 254)];
    signal input iop[25749];
    component stark_verifier = Verify();
    component claim = Journal(n);

    for (var i = 0; i < 25749; i++) {
        stark_verifier.iop[i] <== iop[i];
    }

    for (var i = 0; i < (n \ 254); i++) {
        for (var j = 0; j < 254; j++) {
            claim.journal_bytes_in[254 * i + j] <== input_n2b[i].out[j];
        }
    }
    for (var j = 0; j < (n % 254); j++) {
        claim.journal_bytes_in[254 * (n \ 254) + j] <== input_last_n2b.out[j];
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

component main { public [ journal ] } = VerifyForGuest(32);
