#include <stdio.h>
// #include <jni.h>

#define _stdcall __attribute__((stdcall))

// _stdcall int add(int a, int b) {
//	return a + b;
// }

int massive(int a, int b, int c, int d, int e, int f, int g, int h, int i , int j) {
	return a + b + c + d + e + f + g + h + i + j;
}


int main(int argc, char **argv) {

	int a = 93;
	int b = 12;

	// Random command to keep function from being optimized
	//asm("mov $0 %rax");

	// int result = add(a, b);
	int result = massive(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);

	printf("Got: %d\n", result);




	return 0;
}


