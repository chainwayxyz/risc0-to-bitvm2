pragma circom 2.1.6;

include "../../circomlib/circuits/bitify.circom";

/*
	Common functionality for Blake3. The starting point of this code is from Blake 2: https://github.com/bkomuves/hash-circuits/blob/master/circuits/blake2/blake2_common.circom
*/

//------------------------------------------------------------------------------


/*
	The permutation table where `iO` is the index of the permutation round
	Wait maybe not...
	This is the wrong permutation and corresponds to Blake2s
*/
template Blake3Permute() {
	// TODO: fix up
	signal input inp[16];
  signal output out[16];

  var sigma[16] =
    [2, 6, 3, 10, 7, 0, 4, 13, 1, 11,  12, 5,  9, 14, 15, 8];


  for(var j=0; j<16; j++) { out[j] <== inp[sigma[j]]; }
  // for(var j=0; j<16; j++) { out[sigma[j]] <== inp[j]; }
}


//------------------------------------------------------------------------------
// XOR 3 bits together

template XOR3() {
  signal input  x;
  signal input  y;
  signal input  z;
  signal output out;

  signal tmp <== y*z;
  out <== x * (1 - 2*y - 2*z + 4*tmp) + y + z - 2*tmp;
}

template XOR2() {
  signal input  x;
  signal input  y;
  signal output out;

  // If x = 0, then out = y and vice versa
  // If both are 1, the output 0
	out <== x + y - 2*x*y;
}

//------------------------------------------------------------------------------
// XOR 2 words together

template XorWord2(n) {
  signal input  x;
  signal input  y;

  signal out_bits[n];
  signal output out_word;

  component tb_x = ToBits(n); 
  component tb_y = ToBits(n);

  tb_x.inp <== x;
  tb_y.inp <== y;  

  component xor[n];

  var acc = 0;
  for(var i=0; i<n; i++) { 
    xor[i] = XOR2();
    xor[i].x   <== tb_x.out[i];
    xor[i].y   <== tb_y.out[i];
    xor[i].out ==> out_bits[i];
    acc += out_bits[i] * (2**i);
  }

  out_word <== acc;
}



//------------------------------------------------------------------------------
// XOR 3 words together

template XorWord3(n) {
  signal input  x;
  signal input  y;
  signal input  z;
  signal output out_bits[n];
  signal output out_word;

  component tb_x = ToBits(n); 
  component tb_y = ToBits(n);
  component tb_z = ToBits(n);

  tb_x.inp <== x;
  tb_y.inp <== y;  
  tb_z.inp <== z;

  component xor[n];

  var acc = 0;
  for(var i=0; i<n; i++) { 
    xor[i] = XOR3();
    xor[i].x   <== tb_x.out[i];
    xor[i].y   <== tb_y.out[i];
    xor[i].z   <== tb_z.out[i];
    xor[i].out ==> out_bits[i];
    acc += out_bits[i] * (2**i);
  }

  out_word <== acc;
}

//------------------------------------------------------------------------------
// XOR a word with a constant

template XorWordConst(n, kst_word) {
  signal input  inp_word;
  signal output out_bits[n];
  signal output out_word;

  component tb = ToBits(n);
  tb.inp <== inp_word;

  var acc = 0;
  for(var i=0; i<n; i++) {
    var x = tb.out[i];
    var y = (kst_word >> i) & 1;
    out_bits[i] <== x + y - 2*x*y;
    acc += out_bits[i] * (2**i);
  }

  out_word <== acc;  
}

//------------------------------------------------------------------------------
// decompose an n-bit number into bits

template ToBits(n) {
  signal input  inp;
  signal output out[n];

  var sum = 0;
  for(var i=0; i<n; i++) {
    out[i] <-- (inp >> i) & 1;
    out[i] * (1-out[i]) === 0;
    sum += (1<<i) * out[i];
  }

  inp === sum;
}

//------------------------------------------------------------------------------
// decompose a 33-bit number into the low 32 bits and the remaining 1 bit

// TODO: check me
template Bits33() {
  signal input  inp;
  signal output out_bits[32];
  signal output out_word;
  signal u;

  var sum = 0;
  for(var i=0; i<32; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 32) & 1;
  u*(1-u) === 0;

  inp === sum + (1<<32)*u;
  out_word <== sum;
}

//------------------------------------------------------------------------------
// decompose a 34-bit number into the low 32 bits and the remaining 2 bits

template Bits34() {
  signal input  inp;
  signal output out_bits[32];
  signal output out_word;
  signal u,v;

  var sum = 0;
  for(var i=0; i<32; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 32) & 1;
  v <-- (inp >> 33) & 1;
  u*(1-u) === 0;
  v*(1-v) === 0;

  inp === sum + (1<<32)*u + (1<<33)*v;
  out_word <== sum;
}

//------------------------------------------------------------------------------
// decompose a 65-bit number into the low 64 bits and the remaining 1 bit

template Bits65() {
  signal input  inp;
  signal output out_bits[64];
  signal output out_word;
  signal u;

  var sum = 0;
  for(var i=0; i<64; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 64) & 1;
  u*(1-u) === 0;

  inp === sum + (1<<64)*u;
  out_word <== sum;
}

//------------------------------------------------------------------------------
// decompose a 66-bit number into the low 64 bits and the remaining 2 bit

template Bits66() {
  signal input  inp;
  signal output out_bits[64];
  signal output out_word;
  signal u,v;

  var sum = 0;
  for(var i=0; i<64; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 64) & 1;
  v <-- (inp >> 65) & 1;
  u*(1-u) === 0;
  v*(1-v) === 0;

  inp === sum + (1<<64)*u + (1<<65)*v;
  out_word <== sum;
}



/*
	Circuit for verifying a single chunk pair hash to a Blake3. In particular the code's
  starting point is Blake2s for Circom (https://github.com/bkomuves/hash-circuits/blob/master/circuits/blake2/blake2s.circom)
*/

// Some  notes: TODO:
/*
  We are going to need an adder (on the bits)
  
*/

//------------------------------------------------------------------------------

template IV() {
  signal output out[8];

  var initializationVector[8] = 
    [ 0x6A09E667 , 0xBB67AE85 , 0x3C6EF372 , 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19];

  for(var j=0; j<8; j++) { out[j] <== initializationVector[j]; }
}

//------------------------------------------------------------------------------
// XOR-s two 32-bit vectors and then rotates the result right by the given amount of bits

template RotXorBits(R) {
  signal input  inp1_bits[32];
  signal input  inp2_bits[32];
  signal output out_bits[32];
  signal output out_word;

  signal aux[32];
  for(var i=0; i<32; i++) {
    aux[i] <== inp1_bits[i] + inp2_bits[i] - 2 * inp1_bits[i] * inp2_bits[i];
  }

  var acc = 0;
  for(var i=0; i<32; i++) {
    out_bits[i] <== aux[ (i+R) % 32 ];
    acc += out_bits[i] * (2**i);
  }

  out_word <== acc;
}

//--------------------------------------
// XOR-s a 32-bit word with a bit-vector
// and then rotates the result right by the given amount of bits

template RotXorWordBits(R) {
  signal input  inp1_word;
  signal input  inp2_bits[32];
  signal output out_bits[32];
  signal output out_word;

  component tb = ToBits(32);
  component rx = RotXorBits(R);

  tb.inp    <== inp1_word;
  tb.out    ==> rx.inp1_bits;
  inp2_bits ==> rx.inp2_bits;
  out_bits  <== rx.out_bits;
  out_word  <== rx.out_word;
}

//------------------------------------------------------------------------------

// Should be equivalent to Blake2S
template HalfFunG(a,b,c,d, R1,R2) {
  signal input  v[16];
  signal input  xy;
  signal output out[16];

  for(var i=0; i<16; i++) {
    if ((i!=a) && (i!=b) && (i!=c) && (i!=d)) {
      out[i] <== v[i];
    }
  }

  component add1 = Bits34();        // sum of three words needs 34 bits
  component add3 = Bits33();        // sum of two words only needs 33 bits

  component rxor2 = RotXorWordBits(R1);
  component rxor4 = RotXorWordBits(R2);

  add1.inp      <== v[a] + v[b] + xy;
  v[d]          ==> rxor2.inp1_word;
  add1.out_bits ==> rxor2.inp2_bits;
  add3.inp      <== v[c] + rxor2.out_word;
  v[b]          ==> rxor4.inp1_word;
  add3.out_bits ==> rxor4.inp2_bits;

  out[a] <== add1.out_word;
  out[d] <== rxor2.out_word;
  out[c] <== add3.out_word;
  out[b] <== rxor4.out_word;
}

//------------------------------------------------------------------------------
// the mixing function G

// inputs and output and x,y consists of 32 bit words
template MixFunG(a,b,c,d) {
  signal input  inp[16];
  signal output out[16];
  signal input x;
  signal input y;

  component half1 = HalfFunG(a,b,c,d, 16,12);
  component half2 = HalfFunG(a,b,c,d,  8, 7);

  half1.v   <== inp;
  half1.xy  <== x;
  // TODO: G_half can be made more efficient by only taking in 4 wires, not 16.
  // Pipe the output into half2.v?? TODO: why Shouldn't this be m_{2i + 1} instead. I think yes
  // Wait. Nvmd inp is the v_0, v_1... etc etc..
  half1.out ==> half2.v;
  half2.xy  <== y;
  half2.out ==> out;
}

//------------------------------------------------------------------------------
// a single round

template SingleRound() {
  signal input  inp[16];
  signal input  msg[16];
  signal output out[16];

	// This is in a sep component
  // TODO: we need to do the permutations. 
  // TODO: maybe its a more hardcoded thing?

  component GS[8];

  signal vs[9][16];

  inp ==> vs[0];

	// Okay I think that these Gs are essentially correct. Not the most efficient but good enough
  // Le sigma est quoi?
  GS[0] = MixFunG(  0 ,  4 ,  8 , 12 ) ; GS[0].x <== msg[ 0] ; GS[0].y <== msg[1];
  GS[1] = MixFunG(  1 ,  5 ,  9 , 13 ) ; GS[1].x <== msg[ 2] ; GS[1].y <== msg[3];
  GS[2] = MixFunG(  2 ,  6 , 10 , 14 ) ; GS[2].x <== msg[ 4] ; GS[2].y <== msg[5];
  GS[3] = MixFunG(  3 ,  7 , 11 , 15 ) ; GS[3].x <== msg[ 6] ; GS[3].y <== msg[7]; 
 
  GS[4] = MixFunG(  0 ,  5 , 10 , 15 ) ; GS[4].x <== msg[ 8] ; GS[4].y <== msg[ 9] ;
  GS[5] = MixFunG(  1 ,  6 , 11 , 12 ) ; GS[5].x <== msg[10] ; GS[5].y <== msg[11] ;
  GS[6] = MixFunG(  2 ,  7 ,  8 , 13 ) ; GS[6].x <== msg[12] ; GS[6].y <== msg[13] ;
  GS[7] = MixFunG(  3 ,  4 ,  9 , 14 ) ; GS[7].x <== msg[14] ; GS[7].y <== msg[15] ;

  for(var i=0; i<8; i++) {
    GS[i].inp <== vs[i];
    GS[i].out ==> vs[i+1];
  }

  out <== vs[8];
}

//------------------------------------------------------------------------------
// the compression function F
//
// t is the offset counter
// f should be 1 for the final block and 0 otherwise
//
// TODO: do we need range checks that all the words are 32 bits???
// TODO: MAYBE!
template Blake3Compression() {
  signal input  h[8];         // the state (8 words)
  signal input  m[16];        // the message block (16 words)
  signal input t[2];
  signal input b;
  signal input d;
  
  signal output out[16];       // new state TODO: MAYBE OUTPUT THE WHOLE STATE???

  component iv = IV();
  signal init[16];

	/********* Initialize the state for first round *********/
  for (var i=0; i<8; i++) { init[i  ] <== h[i];      }
  for (var i=0; i<4; i++) { init[i+8] <== iv.out[i]; }
  for (var i=0; i<2; i++) { init[i + 12] <== t[i];   }
  init[14] <== b; init[15] <== d;

  component rounds[7];
  component permuters[6];
	
  // TODO: change name of init...
  rounds[0] = SingleRound();
  rounds[0].msg <== m;
  rounds[0].inp <== init;

 	/********* Perform all rounds. Using `init` for the 1st round and not permuting on the 7th  *********/
  for(var i=0; i<6; i++) {
    rounds[i + 1] = SingleRound();
    permuters[i] = Blake3Permute();

    if (i == 0) {
      m ==> permuters[i].inp;
    } else {
      permuters[i - 1].out ==> permuters[i].inp;
    }
    permuters[i].out ==> rounds[i + 1].msg;
    rounds[i].out ==> rounds[i + 1].inp;
  }
  
  // As per page 6 of the PDF. Output the lower `h'` representing the compressed state
  // TODO: also the remaining 8 wires...
  component outXor[16];
  for(var i=0; i<8; i++) {
    outXor[i] = XorWord2(32);
    outXor[i].x        <== rounds[6].out[i];
    outXor[i].y        <== rounds[6].out[i + 8];
    outXor[i].out_word ==> out[i];
  }
  for (var i = 8; i < 16; i++) {
    outXor[i] = XorWord2(32);
    // Assign the out of round i
    outXor[i].x        <== rounds[6].out[i];
    // Assign the corresponding input i
    outXor[i].y        <== h[i - 8];
    outXor[i].out_word ==> out[i];   
  }
}

/**
TODO: have support for different modes.
We especially care about support for **hash** mode
*/

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