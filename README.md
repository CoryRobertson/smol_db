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
