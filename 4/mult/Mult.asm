// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
// The algorithm is based on repetitive addition.

@0
D=M
@counter
M=D
@2
M=0
(LOOP) @counter
D=M
@12345
D;JEQ
@1
D=M
@2
M=D+M
@counter
M=M-1
@LOOP
0;JMP

