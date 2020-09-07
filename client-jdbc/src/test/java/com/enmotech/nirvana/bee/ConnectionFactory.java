package com.enmotech.nirvana.bee;

import com.enmotech.nirvana.bee.connector.AgentDatasource;
import com.enmotech.nirvana.bee.connector.BeeDatasource;
import com.enmotech.nirvana.bee.connector.RemoteDatasource;

import javax.sql.DataSource;
import java.sql.SQLException;
import java.util.Map;
import java.util.Properties;

public class ConnectionFactory {
    final String ADDR = "127.0.0.1";
    final Integer PORT = 6142;

    int getProxyPort() {
       String port =  System.getenv("RUST_SSH2_FIXTURE_PORT");
       Map<String,String> envs = System.getenv();
       if (port == null){
           return 8022;
       }
       return Integer.parseInt(port);
    }

    public RemoteDatasource createRemoteDatasource(BeeDatasource.SessionMode mode) throws SQLException {
        RemoteDatasource datasource = new RemoteDatasource(ADDR, PORT);
        datasource.setSessionMode(mode);
        datasource.setConnectTimeout(5);
        datasource.authPublicKey(System.getProperty("user.name"));
        datasource.connectionProxy("127.0.0.1", getProxyPort());
        return datasource;
    }

    public AgentDatasource createClientAgentInfo(BeeDatasource.SessionMode mode) {
        AgentDatasource datasource = new AgentDatasource(ADDR, PORT);
        datasource.setSessionMode(mode);
        datasource.setConnectTimeout(5);
        return datasource;
    }
}
