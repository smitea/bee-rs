package com.enmotech.nirvana.bee.connector;

import java.sql.Connection;
import java.sql.Driver;
import java.sql.DriverPropertyInfo;
import java.sql.SQLException;
import java.sql.SQLFeatureNotSupportedException;
import java.util.Properties;
import java.util.logging.Logger;

public class BeeDriver implements Driver {
    @Override
    public Connection connect(String url, Properties info) throws SQLException {
        ClientInfo clientInfo = new ClientInfo(url, info);
        return new BeeConnection(clientInfo);
    }

    @Override
    public boolean acceptsURL(String url) throws SQLException {
        try {
            new ClientInfo(url, new Properties());
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    @Override
    public DriverPropertyInfo[] getPropertyInfo(String url, Properties info) throws SQLException {
        return new DriverPropertyInfo[]{
                new DriverPropertyInfo(ClientInfo.CONNECTION_TIMEOUT, "10"),
                new DriverPropertyInfo(ClientInfo.BEE_HOST, "127.0.0.1"),
                new DriverPropertyInfo(ClientInfo.BEE_PORT, "6142"),
                new DriverPropertyInfo(ClientInfo.SOCKET_TIMEOUT, "20"),
                new DriverPropertyInfo(ClientInfo.APPLICATION, "jdbc"),
                new DriverPropertyInfo(ClientInfo.OS_VERSION, ""),
                new DriverPropertyInfo(ClientInfo.ENVIRONMENTS, "[BETHUNE_PATH: bee]")
        };
    }

    @Override
    public int getMajorVersion() {
        return 0;
    }

    @Override
    public int getMinorVersion() {
        return 1;
    }

    @Override
    public boolean jdbcCompliant() {
        return false;
    }

    @Override
    public Logger getParentLogger() throws SQLFeatureNotSupportedException {
        return Logger.getLogger("bee");
    }
}
