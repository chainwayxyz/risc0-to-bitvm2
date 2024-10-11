pragma circom 2.0.4;

include "../../circomlib/circuits/sha256/sha256.circom";
include "../../circomlib/circuits/bitify.circom";

// Here, we take the journal (commitments of the stark_verify guest) and generate the claim_digest, which corresponds to the out[2], out[3] of the iop.
template Journal() {
    signal input journal_bits_in[256]; // journal in bits
    signal input pre_state_digest_bits[256]; // pre_state_digest in bits, this is kind of needed since it changes each time a different guest circuit is used
    signal output out[2]; // claim_digest in [u128, u128]
    // signal output journal_digest_252[252];
    component claim_hasher = Sha256(1360); // hash(receipt_claim_tag_hash, claim_input_digest, claim_pre_digest, claim_post_digest, claim_output_digest, 00000000000000000400)
    component output_hasher = Sha256(784); // Depends on journal, hash(output_tag_hash, journal_digest, [0u8; 32], 0200)
    // pre_digest = 60890323955764732393576659129306652353038634169504427485921095890475307913310; // Change this depending on the circuit
    // var pre_digest_bits[256] = [1,0,0,0,0,1,1,0,1,0,0,1,1,1,1,0,1,0,1,1,0,0,1,1,1,0,1,1,1,0,0,1,0,1,0,1,0,1,1,1,0,0,1,1,1,0,0,1,1,0,0,0,1,0,1,1,1,1,1,1,0,0,0,1,0,0,1,1,1,0,0,1,1,0,1,1,0,0,1,0,1,1,1,1,0,0,0,1,0,0,1,0,0,0,1,1,1,1,0,1,0,1,1,1,0,1,1,0,0,1,1,0,0,1,1,0,1,0,0,1,1,1,0,1,0,1,1,0,1,0,1,0,1,1,0,0,1,0,0,1,1,1,0,1,1,1,0,1,0,1,1,0,1,1,1,0,0,1,1,1,0,0,0,1,0,1,1,0,0,1,0,0,0,0,1,1,1,1,0,1,0,1,1,0,0,1,1,1,1,1,0,0,1,0,1,0,0,0,0,0,1,0,0,1,0,1,1,1,1,1,0,1,0,0,0,1,1,0,1,0,0,0,0,1,1,0,0,1,1,1,0,0,1,1,1,1,0,1,1,0,0,1,0,1,0,0,0,0,0,1,0,1,1,1,1,0];
    // post_digest = 74032234001928697417723972706442396998535281042410084191249638976189322148066; // Constant no matter the circuit
    var post_digest_bits[256] = [1,0,1,0,0,0,1,1,1,0,1,0,1,1,0,0,1,1,0,0,0,0,1,0,0,1,1,1,0,0,0,1,0,0,0,1,0,1,1,1,0,1,0,0,0,0,0,1,1,0,0,0,1,0,0,1,1,0,0,1,0,1,1,0,0,0,1,1,0,1,0,0,0,0,0,0,1,0,1,1,1,0,0,0,0,1,0,0,1,1,1,0,0,1,0,1,1,0,1,0,1,0,0,1,0,0,0,0,1,1,1,1,0,0,1,1,1,1,1,0,1,1,1,1,0,1,0,0,1,1,0,0,0,1,0,0,1,0,0,1,1,1,0,1,0,0,1,0,0,0,1,0,1,1,0,0,0,1,1,1,1,0,0,1,1,1,1,0,0,1,0,0,0,1,0,0,1,0,1,0,1,0,1,0,1,1,0,1,1,0,0,0,0,0,1,0,0,0,1,0,1,1,1,0,1,1,0,0,1,0,0,1,1,1,0,0,0,0,1,1,0,0,0,1,0,0,1,1,1,1,1,0,0,0,0,1,1,1,1,0,1,0,1,1,1,0,0,0,1,1,1,0,0,0,1,0];
    var output_tag_bits[256] = [0,1,1,1,0,1,1,1,1,1,1,0,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1,0,1,0,0,1,1,1,1,0,0,0,1,0,1,1,0,1,0,0,0,1,1,1,0,1,1,1,0,1,0,0,0,1,1,1,1,1,0,1,1,1,1,0,0,0,0,0,1,1,0,1,0,1,1,1,1,0,1,1,1,0,1,1,0,0,0,1,0,1,1,1,0,1,1,0,0,0,1,0,1,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,1,0,1,1,1,1,1,1,1,1,1,0,1,0,1,0,1,0,1,0,1,1,0,0,1,0,0,1,0,0,0,1,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,1,0,0,1,1,0,1,0,0,1,0,1,1,0,1,1,1,1,1,0,0,1,1,0,0,0,1,1,1,1,0,1,1,0,1,0,0,0,1,1,0,0,1,0,1,1,0,1,0,0,1,1,0,1,0,1,0,1,0,1,1,0,0,1,1,1,0,1,0,1,0,0];
    // output_tag = 54240429074780157751094335427642025272505087342533963631932091736940340402644; // Constant = hash("risc0.Output")

    component journal_hasher = Sha256(256); // Depends on journal, hash(journal)

    log("journal_bits");
    for (var i = 0; i < 256; i++) {
        log(journal_bits_in[i]);
    }
    
    journal_hasher.in <== journal_bits_in;

    log("journal_digest");
    for (var i = 0; i < 256; i++) {
        log(journal_hasher.out[i]);
    }
    
    for (var i = 0; i < 256; i++) {
        output_hasher.in[i] <== output_tag_bits[i];
    }
    for(var i = 0; i < 256; i++) {
        output_hasher.in[256 + i] <== journal_hasher.out[i];
    }
    for (var i = 0; i < 256; i++) {
        output_hasher.in[512 + i] <== 0;
    }

    output_hasher.in[768] <== 0;
    output_hasher.in[769] <== 0;
    output_hasher.in[770] <== 0;
    output_hasher.in[771] <== 0;
    output_hasher.in[772] <== 0;
    output_hasher.in[773] <== 0;
    output_hasher.in[774] <== 1;
    output_hasher.in[775] <== 0;
    output_hasher.in[776] <== 0;
    output_hasher.in[777] <== 0;
    output_hasher.in[778] <== 0;
    output_hasher.in[779] <== 0;
    output_hasher.in[780] <== 0;
    output_hasher.in[781] <== 0;
    output_hasher.in[782] <== 0;
    output_hasher.in[783] <== 0;

    var receipt_claim_tag_bits[256] = [1,1,0,0,1,0,1,1,0,0,0,1,1,1,1,1,1,1,1,0,1,1,1,1,1,1,0,0,1,1,0,1,0,0,0,1,1,1,1,1,0,0,1,0,1,1,0,1,1,0,0,1,1,0,1,0,0,1,1,0,0,1,0,0,1,0,0,1,0,1,1,1,0,1,0,1,1,1,0,0,1,0,1,1,1,0,1,1,1,0,1,1,1,1,1,1,0,1,1,0,1,1,1,0,0,0,0,1,0,1,1,0,0,0,0,1,1,1,1,0,0,0,1,0,1,0,0,1,0,0,0,1,0,1,0,0,0,1,0,0,0,0,1,1,0,1,0,0,1,0,1,1,0,0,0,0,1,1,0,0,1,0,1,1,1,0,1,1,1,0,0,1,1,0,0,1,0,1,1,0,0,0,0,0,1,0,1,1,1,0,0,0,0,1,0,0,1,1,0,1,1,1,1,1,0,1,0,1,1,1,0,1,0,1,1,1,0,0,0,1,0,1,1,1,1,1,1,0,1,0,0,0,0,1,1,0,1,0,1,1,0,1,0,0,1,0,0,0,1,0,1,0,1,1,1,1];

    for (var i = 0; i < 256; i++) {
        claim_hasher.in[i] <== receipt_claim_tag_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[256 + i] <== 0; // Input digest, constant no matter the circuit
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[512 + i] <== pre_state_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[768 + i] <== post_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[1024 + i] <== output_hasher.out[i];
    }
    for (var i = 0; i < 64; i++) {
        claim_hasher.in[1280 + i] <== 0;
    }

    claim_hasher.in[1344] <== 0;
    claim_hasher.in[1345] <== 0;
    claim_hasher.in[1346] <== 0;
    claim_hasher.in[1347] <== 0;
    claim_hasher.in[1348] <== 0;
    claim_hasher.in[1349] <== 1;
    claim_hasher.in[1350] <== 0;
    claim_hasher.in[1351] <== 0;
    claim_hasher.in[1352] <== 0;
    claim_hasher.in[1353] <== 0;
    claim_hasher.in[1354] <== 0;
    claim_hasher.in[1355] <== 0;
    claim_hasher.in[1356] <== 0;
    claim_hasher.in[1357] <== 0;
    claim_hasher.in[1358] <== 0;
    claim_hasher.in[1359] <== 0;

    log("sha digest");
    for (var i = 0; i < 256; i++) {
        log(claim_hasher.out[i]);
    }

    component b2n_1 = Bits2Num(128);
    for(var i = 0; i < 16; i++) {
        for(var j = 0; j < 8; j++) {
            b2n_1.in[8 * i + 7 - j] <== claim_hasher.out[8 * i + j];
        }
    }
    out[0] <== b2n_1.out;

    component b2n_2 = Bits2Num(128);
    for(var i = 0; i < 16; i++) {
        for(var j = 0; j < 8; j++) {
            b2n_2.in[8 * i + 7 - j] <== claim_hasher.out[128 + 8 * i + j];
        }
    }
    out[1] <== b2n_2.out;
}