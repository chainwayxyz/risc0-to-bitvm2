// SPDX-License-Identifier: GPL-3.0
/*
    Copyright 2021 0KIMS association.

    This file is generated with [snarkJS](https://github.com/iden3/snarkjs).

    snarkJS is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    snarkJS is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with snarkJS. If not, see <https://www.gnu.org/licenses/>.
*/

pragma solidity >=0.7.0 <0.9.0;

contract Groth16Verifier {
    // Scalar field size
    uint256 constant r    = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    // Base field size
    uint256 constant q   = 21888242871839275222246405745257275088696311157297823662689037894645226208583;

    // Verification Key data
    uint256 constant alphax  = 20491192805390485299153009773594534940189261866228447918068658471970481763042;
    uint256 constant alphay  = 9383485363053290200918347156157836566562967994039712273449902621266178545958;
    uint256 constant betax1  = 4252822878758300859123897981450591353533073413197771768651442665752259397132;
    uint256 constant betax2  = 6375614351688725206403948262868962793625744043794305715222011528459656738731;
    uint256 constant betay1  = 21847035105528745403288232691147584728191162732299865338377159692350059136679;
    uint256 constant betay2  = 10505242626370262277552901082094356697409835680220590971873171140371331206856;
    uint256 constant gammax1 = 11559732032986387107991004021392285783925812861821192530917403151452391805634;
    uint256 constant gammax2 = 10857046999023057135944570762232829481370756359578518086990519993285655852781;
    uint256 constant gammay1 = 4082367875863433681332203403145435568316851327593401208105741076214120093531;
    uint256 constant gammay2 = 8495653923123431417604973247489272438418190587263600148770280649306958101930;
    uint256 constant deltax1 = 5498479339940192123564205377940229715613413839689031307888939901996329188818;
    uint256 constant deltax2 = 21732635929337488520552142423624721233615023579132141095032144318250021700550;
    uint256 constant deltay1 = 14001324242144111265304429679270067298157681596990202266113441439943565135135;
    uint256 constant deltay2 = 7420944059007423931385590858010128290267285786104220588672477460984509999251;

    
    uint256 constant IC0x = 5700033550455073695366774296857389188375816473851990765983224467808393043972;
    uint256 constant IC0y = 6975554180561957848831621218765426435198099269133292312327855722856422038760;
    
    uint256 constant IC1x = 4559303082298703622161804195264032180950754936339678100276042350461046432919;
    uint256 constant IC1y = 2075464936578610564154843822110322532382895115508765842584014130928833412991;
    
    uint256 constant IC2x = 14174992732249746808528672100012717190524276801888985453239203998252605784685;
    uint256 constant IC2y = 13555323685437037318149764982158995606465513198273300885849800983643814434846;
    
    uint256 constant IC3x = 4271701481949999450683766708131503276130202028392028597937587447038930595864;
    uint256 constant IC3y = 17198171454193344848209651592689926225978259032690478432358788677879404056459;
    
    uint256 constant IC4x = 11801535475612142741690820628906755468063118313987010427737559204649153282477;
    uint256 constant IC4y = 7250167606487183048435028699287186850445789639088853700599521322521218339550;
    
    uint256 constant IC5x = 15417267448275705336728404519854681618778046079903610944414126744768989190411;
    uint256 constant IC5y = 857849787111026313209062675522502664459387427347319869569844691782243652842;
    
    uint256 constant IC6x = 5349756772371386581303783156655639688863347537880574211045339628123788644320;
    uint256 constant IC6y = 10972831661344103419581641349254353417378037801089581355914078825722225868470;
    
    uint256 constant IC7x = 10056224547553983585036958109785316270139178450504316545046450338875290676943;
    uint256 constant IC7y = 3544471654383389502091174434432778491993385493091120319222854987802712245010;
    
    uint256 constant IC8x = 2627629058551014121930455416080243769750085359631739882228087034889284110341;
    uint256 constant IC8y = 10135281447990311956380884591392495669143778696905351997000273844315699894070;
    
    uint256 constant IC9x = 9516829308194152033058412549711109958639953291388027478503996375488056930397;
    uint256 constant IC9y = 19257438088673461751263098944327875558214632726364451336051423810016496171440;
    
    uint256 constant IC10x = 3972670360101836753304053013409082551170691164871378800539600310925359215651;
    uint256 constant IC10y = 19075340887618953376406977032868672975188019909742530261347105926965494157250;
    
    uint256 constant IC11x = 2231411951853426944266114871264951427081626928118104584049402858123907327796;
    uint256 constant IC11y = 17161417533043431125871834042695560400002651385201830184748953484340467357915;
    
    uint256 constant IC12x = 18401320671755409264240074796913716481958328379092382035606106805838203934341;
    uint256 constant IC12y = 18893111009781309429154803527123708657515110077275230186151269441098581836324;
    
    uint256 constant IC13x = 1215440900141739945539601124926696685874831093948986730254881095576326201632;
    uint256 constant IC13y = 5561514034344440862745180240515120540477033743300510909089131189281407815478;
    
    uint256 constant IC14x = 18620175185132281176432742829730864893077874451643492681279294407314699920342;
    uint256 constant IC14y = 8860374322436523173678901458327587024790826098043616827077908111773332296907;
    
    uint256 constant IC15x = 7080632882697841604960909566396113259509897432852070912131549531876315817734;
    uint256 constant IC15y = 3425426415347299169936550240969291372481984933093418240304111807924886647683;
    
    uint256 constant IC16x = 15932827180859129635485548175191034824852659642278682171462569050655629422973;
    uint256 constant IC16y = 19494611418141575022444760070683204279465310686006176881225178493017344930488;
    
    uint256 constant IC17x = 14251704090404245982546405057344610134053414114450087643430249077404735183710;
    uint256 constant IC17y = 12126906136928063813190166633177975380776143395555147476411578278663663245592;
    
    uint256 constant IC18x = 12187526007334512378904195735715981910623864370537736031179946869509353126899;
    uint256 constant IC18y = 1856383540150509342271278688236496848557489058752863680167593445062894425368;
    
    uint256 constant IC19x = 12707620580981475313285537837681068575145761867680173222276079993058333940797;
    uint256 constant IC19y = 9827357576725928593237830362469530801222842595869862689199762161036132725042;
    
    uint256 constant IC20x = 6623953735015209213503325375609752630187616768560860645979787191679106496442;
    uint256 constant IC20y = 8581283046898274645755157785299862067896486450836027110824126808881844374110;
    
    uint256 constant IC21x = 1362494365587895499789703933920673995384172924730749213683594767983887207640;
    uint256 constant IC21y = 18840784618217931199758147839838405862707360395380979459679470533658338889411;
    
    uint256 constant IC22x = 11724639300278741050295788383186020356426050673780266478105187328318094358556;
    uint256 constant IC22y = 8014470574714959403612613957827677556259346385034441414459204965618103945935;
    
    uint256 constant IC23x = 6575714383643352502611845598900857489170877316810343792885918580192463653066;
    uint256 constant IC23y = 8128130409470850673923307623870690039935163770258100612882562429139180341203;
    
    uint256 constant IC24x = 16024679446954145669673056672913792458641596043657349535636788385080714472808;
    uint256 constant IC24y = 5633122142609207226088714386512802771848200020767670070727446295988440748602;
    
    uint256 constant IC25x = 12782850020587501960032798791348665362164859828552506127590644985594372193234;
    uint256 constant IC25y = 8742657115902961171301729021947552870740137635830955764260402032773948397329;
    
    uint256 constant IC26x = 13592620293616850083105516779167554124564416975663829563069167939526239895588;
    uint256 constant IC26y = 21456913563428558085031206801976129013985811844340885281786433402808342088760;
    
    uint256 constant IC27x = 19019050989906942876685849135555958510304598748230745292751851539937050193943;
    uint256 constant IC27y = 14853167062384201380027368118824833660549734837650042079375937854741350620973;
    
    uint256 constant IC28x = 10582495285155395702417190625395247343774365553526441897023709430917678107920;
    uint256 constant IC28y = 6369782619517557260749322864323946357368856838177179643059247010653298532306;
    
    uint256 constant IC29x = 17007509338247864920537336982323142115271094405841722269392341905261661020108;
    uint256 constant IC29y = 21599446408071649010897139493552203862739673969519544197449231863443562576508;
    
    uint256 constant IC30x = 9058049989990844788885494158931973621335416496286130285269591147022372407858;
    uint256 constant IC30y = 13961975721809416482476888001187108509270753990460441378638656014460371402866;
    
    uint256 constant IC31x = 19774466262631826424136750082869813112388536586383592445670427331750984346475;
    uint256 constant IC31y = 5792635842216632033938403791861929505817215143671433309792752678701675388511;
    
    uint256 constant IC32x = 21804607041180251892516444208724939445245288342999077373735894893517469998499;
    uint256 constant IC32y = 12881483835378170939190674133273921793329203913537641124899890436082454996987;
    
 
    // Memory data
    uint16 constant pVk = 0;
    uint16 constant pPairing = 128;

    uint16 constant pLastMem = 896;

    function verifyProof(uint[2] calldata _pA, uint[2][2] calldata _pB, uint[2] calldata _pC, uint[32] calldata _pubSignals) public view returns (bool) {
        assembly {
            function checkField(v) {
                if iszero(lt(v, r)) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }
            
            // G1 function to multiply a G1 value(x,y) to value in an address
            function g1_mulAccC(pR, x, y, s) {
                let success
                let mIn := mload(0x40)
                mstore(mIn, x)
                mstore(add(mIn, 32), y)
                mstore(add(mIn, 64), s)

                success := staticcall(sub(gas(), 2000), 7, mIn, 96, mIn, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }

                mstore(add(mIn, 64), mload(pR))
                mstore(add(mIn, 96), mload(add(pR, 32)))

                success := staticcall(sub(gas(), 2000), 6, mIn, 128, pR, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }

            function checkPairing(pA, pB, pC, pubSignals, pMem) -> isOk {
                let _pPairing := add(pMem, pPairing)
                let _pVk := add(pMem, pVk)

                mstore(_pVk, IC0x)
                mstore(add(_pVk, 32), IC0y)

                // Compute the linear combination vk_x
                
                g1_mulAccC(_pVk, IC1x, IC1y, calldataload(add(pubSignals, 0)))
                
                g1_mulAccC(_pVk, IC2x, IC2y, calldataload(add(pubSignals, 32)))
                
                g1_mulAccC(_pVk, IC3x, IC3y, calldataload(add(pubSignals, 64)))
                
                g1_mulAccC(_pVk, IC4x, IC4y, calldataload(add(pubSignals, 96)))
                
                g1_mulAccC(_pVk, IC5x, IC5y, calldataload(add(pubSignals, 128)))
                
                g1_mulAccC(_pVk, IC6x, IC6y, calldataload(add(pubSignals, 160)))
                
                g1_mulAccC(_pVk, IC7x, IC7y, calldataload(add(pubSignals, 192)))
                
                g1_mulAccC(_pVk, IC8x, IC8y, calldataload(add(pubSignals, 224)))
                
                g1_mulAccC(_pVk, IC9x, IC9y, calldataload(add(pubSignals, 256)))
                
                g1_mulAccC(_pVk, IC10x, IC10y, calldataload(add(pubSignals, 288)))
                
                g1_mulAccC(_pVk, IC11x, IC11y, calldataload(add(pubSignals, 320)))
                
                g1_mulAccC(_pVk, IC12x, IC12y, calldataload(add(pubSignals, 352)))
                
                g1_mulAccC(_pVk, IC13x, IC13y, calldataload(add(pubSignals, 384)))
                
                g1_mulAccC(_pVk, IC14x, IC14y, calldataload(add(pubSignals, 416)))
                
                g1_mulAccC(_pVk, IC15x, IC15y, calldataload(add(pubSignals, 448)))
                
                g1_mulAccC(_pVk, IC16x, IC16y, calldataload(add(pubSignals, 480)))
                
                g1_mulAccC(_pVk, IC17x, IC17y, calldataload(add(pubSignals, 512)))
                
                g1_mulAccC(_pVk, IC18x, IC18y, calldataload(add(pubSignals, 544)))
                
                g1_mulAccC(_pVk, IC19x, IC19y, calldataload(add(pubSignals, 576)))
                
                g1_mulAccC(_pVk, IC20x, IC20y, calldataload(add(pubSignals, 608)))
                
                g1_mulAccC(_pVk, IC21x, IC21y, calldataload(add(pubSignals, 640)))
                
                g1_mulAccC(_pVk, IC22x, IC22y, calldataload(add(pubSignals, 672)))
                
                g1_mulAccC(_pVk, IC23x, IC23y, calldataload(add(pubSignals, 704)))
                
                g1_mulAccC(_pVk, IC24x, IC24y, calldataload(add(pubSignals, 736)))
                
                g1_mulAccC(_pVk, IC25x, IC25y, calldataload(add(pubSignals, 768)))
                
                g1_mulAccC(_pVk, IC26x, IC26y, calldataload(add(pubSignals, 800)))
                
                g1_mulAccC(_pVk, IC27x, IC27y, calldataload(add(pubSignals, 832)))
                
                g1_mulAccC(_pVk, IC28x, IC28y, calldataload(add(pubSignals, 864)))
                
                g1_mulAccC(_pVk, IC29x, IC29y, calldataload(add(pubSignals, 896)))
                
                g1_mulAccC(_pVk, IC30x, IC30y, calldataload(add(pubSignals, 928)))
                
                g1_mulAccC(_pVk, IC31x, IC31y, calldataload(add(pubSignals, 960)))
                
                g1_mulAccC(_pVk, IC32x, IC32y, calldataload(add(pubSignals, 992)))
                

                // -A
                mstore(_pPairing, calldataload(pA))
                mstore(add(_pPairing, 32), mod(sub(q, calldataload(add(pA, 32))), q))

                // B
                mstore(add(_pPairing, 64), calldataload(pB))
                mstore(add(_pPairing, 96), calldataload(add(pB, 32)))
                mstore(add(_pPairing, 128), calldataload(add(pB, 64)))
                mstore(add(_pPairing, 160), calldataload(add(pB, 96)))

                // alpha1
                mstore(add(_pPairing, 192), alphax)
                mstore(add(_pPairing, 224), alphay)

                // beta2
                mstore(add(_pPairing, 256), betax1)
                mstore(add(_pPairing, 288), betax2)
                mstore(add(_pPairing, 320), betay1)
                mstore(add(_pPairing, 352), betay2)

                // vk_x
                mstore(add(_pPairing, 384), mload(add(pMem, pVk)))
                mstore(add(_pPairing, 416), mload(add(pMem, add(pVk, 32))))


                // gamma2
                mstore(add(_pPairing, 448), gammax1)
                mstore(add(_pPairing, 480), gammax2)
                mstore(add(_pPairing, 512), gammay1)
                mstore(add(_pPairing, 544), gammay2)

                // C
                mstore(add(_pPairing, 576), calldataload(pC))
                mstore(add(_pPairing, 608), calldataload(add(pC, 32)))

                // delta2
                mstore(add(_pPairing, 640), deltax1)
                mstore(add(_pPairing, 672), deltax2)
                mstore(add(_pPairing, 704), deltay1)
                mstore(add(_pPairing, 736), deltay2)


                let success := staticcall(sub(gas(), 2000), 8, _pPairing, 768, _pPairing, 0x20)

                isOk := and(success, mload(_pPairing))
            }

            let pMem := mload(0x40)
            mstore(0x40, add(pMem, pLastMem))

            // Validate that all evaluations ∈ F
            
            checkField(calldataload(add(_pubSignals, 0)))
            
            checkField(calldataload(add(_pubSignals, 32)))
            
            checkField(calldataload(add(_pubSignals, 64)))
            
            checkField(calldataload(add(_pubSignals, 96)))
            
            checkField(calldataload(add(_pubSignals, 128)))
            
            checkField(calldataload(add(_pubSignals, 160)))
            
            checkField(calldataload(add(_pubSignals, 192)))
            
            checkField(calldataload(add(_pubSignals, 224)))
            
            checkField(calldataload(add(_pubSignals, 256)))
            
            checkField(calldataload(add(_pubSignals, 288)))
            
            checkField(calldataload(add(_pubSignals, 320)))
            
            checkField(calldataload(add(_pubSignals, 352)))
            
            checkField(calldataload(add(_pubSignals, 384)))
            
            checkField(calldataload(add(_pubSignals, 416)))
            
            checkField(calldataload(add(_pubSignals, 448)))
            
            checkField(calldataload(add(_pubSignals, 480)))
            
            checkField(calldataload(add(_pubSignals, 512)))
            
            checkField(calldataload(add(_pubSignals, 544)))
            
            checkField(calldataload(add(_pubSignals, 576)))
            
            checkField(calldataload(add(_pubSignals, 608)))
            
            checkField(calldataload(add(_pubSignals, 640)))
            
            checkField(calldataload(add(_pubSignals, 672)))
            
            checkField(calldataload(add(_pubSignals, 704)))
            
            checkField(calldataload(add(_pubSignals, 736)))
            
            checkField(calldataload(add(_pubSignals, 768)))
            
            checkField(calldataload(add(_pubSignals, 800)))
            
            checkField(calldataload(add(_pubSignals, 832)))
            
            checkField(calldataload(add(_pubSignals, 864)))
            
            checkField(calldataload(add(_pubSignals, 896)))
            
            checkField(calldataload(add(_pubSignals, 928)))
            
            checkField(calldataload(add(_pubSignals, 960)))
            
            checkField(calldataload(add(_pubSignals, 992)))
            
            checkField(calldataload(add(_pubSignals, 1024)))
            

            // Validate all evaluations
            let isValid := checkPairing(_pA, _pB, _pC, _pubSignals, pMem)

            mstore(0, isValid)
             return(0, 0x20)
         }
     }
 }
