package com.enmotech.nirvana.bee.connector;

import java.sql.Connection;
import java.sql.Driver;
import java.sql.DriverPropertyInfo;
import java.sql.SQLException;
import java.sql.SQLFeatureNotSupportedException;
import java.util.Enumeration;
import java.util.Properties;
import java.util.logging.Logger;

public class BeeDriver implements Driver {
    @Override
    public Connection connect(String url, Properties info) throws SQLException {
        ClientInfo clientInfo = new ClientInfo(url, info);
        getParentLogger().info("connect url :" + url);
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
        int size = ClientInfo.DEFAULT_PROPERTIES.size();
        Enumeration<Object> keys = ClientInfo.DEFAULT_PROPERTIES.keys();
        DriverPropertyInfo[] infos = new DriverPropertyInfo[size];
        for (int i = 0; i < size; i++) {
            String key = (String) keys.nextElement();
            String value = ClientInfo.DEFAULT_PROPERTIES.getProperty(key);
            infos[i] = new DriverPropertyInfo(key, value);
        }
        return infos;
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
