#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


struct FFISmolDBClient;


extern "C" {

void smol_db_client_free(FFISmolDBClient *client_ptr);

SmolDbClient *smol_db_client_new(const char *ip);

void smol_db_client_set_key(FFISmolDBClient *client_ptr, const char *key_ptr);

uint8_t *smol_db_client_write_db(FFISmolDBClient *client_ptr,
                                 const char *name,
                                 const char *location,
                                 const char *data);

} // extern "C"
