#include<stdlib.h>
#include<stdint.h>
#include<stdio.h>

uint8_t *pool_alloc(size_t size)
{
	uint8_t *ptr;
	ptr = (uint8_t *)malloc(size+sizeof(unsigned long));
	*(unsigned long *)ptr = size;
	return ptr + sizeof(unsigned long);
}

void pool_free(uint8_t *ptr)
{
	ptr -= sizeof(unsigned long);
	free(ptr);
}
