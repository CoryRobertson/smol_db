//! Library contain the structs that manage the client to connect to smol_db

//TODO: write a smol_db_client struct that facilitates all actions, as abstract as possible. It should be created using a factory function that takes in the desired ip address.
//  The struct should contain a tcp socket, the previously input ip address. These should all be non-public, and everything relating to these objects should be fully wrapped.
//  It should maintain the connection, and allow for abstract functions like:
//  create_db()
//  delete_db()
//  write_db()
//  read_db()
