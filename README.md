![Image of program, small fat cat laying on a server computer](https://raw.githubusercontent.com/CoryRobertson/smol_db/main/images/program_image.png)
### Program art made by [Crisis](https://kikicat.carrd.co/).

# smol_db
A database client and server designed for small databases that simply need quick setup and deletion, read and write.
The goal of this database is to use it in most of my other projects when applicable, and make it as easy to use as possible. The database's structure is simply a hashmap.

### This project consists of 4 subprojects:
- **smol_db_server**:
A server program that waits for connections to it, and serves them on port 8222. It also handles the files needed to run the database.
- **smol_db_client**: 
A library that can be used to interface with the server program.
- **smol_db_common**:
A library used to run a server, should the smol_db_server not be adequate, this library consists of everything necessary to build a server that handles requests and process them.
- **smol_db_viewer**:
An example program that allows the user to connect to a smol_db_server, the program can connect, view, create, delete, read, and write databases on a given server.

### Programs currently using this database:
- [cr_tiler_rs](https://github.com/CoryRobertson/cr_tiler_rs) uses the database to store leaderboards information for the game service.
- Feel free to let me know if you use this database, I would love to know! :)

### Security:
smol_db is not designed to be extremely secure, most of its use cases are exist on the local network, where security can less necessary. 
If there are any improvements that can be made to security that come to my mind, I will slowly implement them as I get around to those ideas.
access keys are not stored in a hash or encrypted format, and therefore should not be assumed to be safe or secure when stored.

### Example Docker-Compose entry
```
db:
    build: https://github.com/CoryRobertson/smol_db.git#main
    image: smol_db_server
    ports:
      - "8222:8222"
    container_name: "smol_db_server_instance1"
    restart: unless-stopped
    volumes:
      - "./smol_db:/data"
```

### Setup:
To create a smol_db_server instance, the above docker compose example can be used, 
or the server package can be built from source and run on the server computer.
After creating an instance of the server on either bare-metal or a docker container, 
simply connect to it using the smol_db_client library, or through the smol_db_viewer.
Images below outline what the smol_db_viewer looks like and what screens are available.

## Example usage of client library:
```rust
use smol_db_client::SmolDbClient;
fn main() {
    // server is assumed to be running on localhost on port 8222
    let mut client = SmolDbClient::new("localhost:8222").unwrap();
    let data = "super cool user data";
    
    let _ = client.set_access_key("readme_db_key".to_string()).unwrap();
    let _ = client.create_db("cool_db_name", DBSettings::default()).unwrap();
    let _ = client.write_db("cool_db_name", "cool_data_location", data).unwrap();

    match client.read_db("cool_db_name","cool_data_location") {
        SuccessReply(response_data) => {
            assert_eq!(&response_data, data);
        }
        SuccessNoData => {
            assert!(false);
        }
    }
}
```

### Connecting:
![Image of connecting to a database using the viewing application](https://raw.githubusercontent.com/CoryRobertson/smol_db/main/images/viewer_connect.png)
### Setting the clients access key:
![Image of setting the key of the client in the viewing application](https://raw.githubusercontent.com/CoryRobertson/smol_db/main/images/viewer_set_key.png)
### Creating a database:
![Image of creating a database using the viewing application](https://raw.githubusercontent.com/CoryRobertson/smol_db/main/images/viewer_create_db.png)
### Viewing the data, and editing the data on the database:
![Image of viewing the data on the database, and also writing to it, using the viewing application](https://raw.githubusercontent.com/CoryRobertson/smol_db/main/images/viewer_data_viewing.png)
### Changing the settings of a database
![Image of changing the database settings using the viewing application](https://raw.githubusercontent.com/CoryRobertson/smol_db/main/images/viewer_db_settings.png)
