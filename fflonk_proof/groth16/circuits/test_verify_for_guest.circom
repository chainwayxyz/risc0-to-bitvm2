pragma circom 2.0.4;

include "test_stark_verify.circom";
include "test_journal.circom";

template VerifyForGuest(n) {
    signal input journal[((n + 251) \ 252)];
    signal input pre_state_digest_bits[256];
    component input_n2b[(n \ 252)];

    for (var i = 0; i < (n \ 252); i++) {
        input_n2b[i] = Num2Bits(252);
        input_n2b[i].in <== journal[i];
    }
    signal input iop[25749];
    component stark_verifier = Verify();
    component claim = Journal(n);

    for (var i = 0; i < 25749; i++) {
        stark_verifier.iop[i] <== iop[i];
    }

    for (var i = 0; i < (n \ 252); i++) {
        for (var j = 0; j < 252; j++) {
            claim.journal_bytes_in[252 * i + j] <== input_n2b[i].out[j];
        }
    }
    if (n % 252 > 0) {
        component input_last_n2b = Num2Bits((n % 252));
        input_last_n2b.in <== journal[(n \ 252)];
        for (var j = 0; j < (n % 252); j++) {
            claim.journal_bytes_in[252 * (n \ 252) + j] <== input_last_n2b.out[j];
        }
    }
    for (var i = 0; i < 256; i++) {
        claim.pre_state_digest_bits[i] <== pre_state_digest_bits[i];
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

component main { public [ journal ] } = VerifyForGuest(2592);
