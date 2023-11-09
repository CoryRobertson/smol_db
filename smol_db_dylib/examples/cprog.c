// this is an example program that uses the ffi library
#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
#include "../../bindings.h"
//typedef struct smol_db_client smol_db_client_t;
//
//extern smol_db_client_t* smol_db_client_new(const char *ip);
//
//extern void smol_db_client_set_key(smol_db_client_t *,const char *key);
//
//extern const char* smol_db_client_free(smol_db_client_t *);
//
//extern const char* smol_db_client_write_db(smol_db_client_t *, const char *db_name,const char *db_location,const char *db_data);

// gcc -o cprog cprog.c -lsmol_db_dylib -L.
int main(void) {
    printf("testing ffi\n");
	FFISmolDBClient *client = smol_db_client_new("localhost:8222");

	smol_db_client_set_key(client,"test_key_123");
	
	smol_db_client_free(client);
	printf("freed client for test\n");
}
