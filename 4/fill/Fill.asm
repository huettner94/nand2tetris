// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

@SCREEN
D=A
@offset
M=D

(LOOP)
@KBD
D=M
@CLEAR
D;JEQ

(FILL)
@offset
D=M
@KBD
D=A-D
@FILL_TODO
D;JGT
@LOOP
0;JMP

(FILL_TODO)
@offset
A=M
M=-1
D=A+1
@offset
M=D
@LOOP
0;JMP 


(CLEAR)
@offset
D=M
@SCREEN
D=A-D
@CLEAR_TODO
D;JNE
@SCREEN
M=0
@LOOP
0;JMP 



(CLEAR_TODO)
@offset
A=M
M=0
D=A-1
@offset
M=D
@LOOP
0;JMP
