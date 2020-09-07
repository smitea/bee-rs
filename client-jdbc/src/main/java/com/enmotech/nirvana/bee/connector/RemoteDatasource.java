package com.enmotech.nirvana.bee.connector;

class RemoteDatasource extends BeeDatasource {
    public RemoteDatasource(String host, int port) {
        super(host, port);
        setDataSourceMode("remote");
        setResource("bee");
    }

    public void connectionProxy(String host, int port){
        setProxyHost(host);
        setProxyPort(port);
    }

    public void authPassword(String username, String password) {
        setUsername(username);
        setPassword(password);
        setConnectionMode("password");
    }

    public void authPublicKey(String username) {
        setUsername(username);
        setConnectionMode("pubkey");
    }
}
