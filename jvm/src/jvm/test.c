#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

extern uint64_t exec_x86_with_stack(void *fn, void *rbp, void *rsp);


int add(int a, int b) {
	printf("Received a: %d (%x) b: %d (%x)\n", a, a, b, b);
	return a + b;
}



int main(int argc, char **argv) {

	uint64_t stack[7];
	stack[0] = 7;
	stack[1] = 13;
	stack[2] = 17;
	stack[3] = 23;
	stack[4] = 27;
	stack[5] = 49;
	stack[6] = 57;

	void *rsp = &stack[0];
	void *rbp = &stack[7];

	printf("rsp: %p\n", rsp);
	printf("rbp: %p\n", rbp);
	printf("Performing operation...\n");

	uint64_t output = exec_x86_with_stack(add, rbp, rsp);

	printf("Finished performing operation!\n");
	printf("Got output of %ld\n", output);




	return 0;
}
