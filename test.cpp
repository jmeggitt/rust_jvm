#include <stdio.h>


int __stdcall add(int, int);

int __stdcall add(int a, int b) {
	return a + b;
}






int main(int argc, char **argv) {

	int a = 93;
	int b = 12;

	// Random command to keep function from being optimized
	asm("mov $0 %rax");

	int result = add(a, b);

	printf("Got: %d\n", result);




	return 0;
}


