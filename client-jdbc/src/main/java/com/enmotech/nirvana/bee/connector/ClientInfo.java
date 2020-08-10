package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;

import java.util.Properties;
import java.util.Set;

public class ClientInfo {
    public static String USERNAME = "username";
    public static String PASSWORD = "password";
    public static String CONNECTION_MODE = "connect_mode";
    public static String SESSION_MODE = "session_mode";
    public static String DATASOURCE_MODE = "datasource_mode";
    public static String CONNECTION_HOST = "connection_host";
    public static String CONNECTION_PORT = "connection_port";
    public static String CONNECTION_RESOURCE = "connection_resource";
    public static String CONNECTION_TIMEOUT = "connection_timeout";
    public static String APPLICATION = "application";
    public static String PUBLIC_KEY = "public_key";
    public static String DEFAULT_CONNECT_MODE = "default";

    private final String beeHost;
    private final int beePort;
    private Properties properties;

    public ClientInfo(String beeHost, int beePort, Properties properties) {
        this.beeHost = beeHost;
        this.beePort = beePort;
        this.properties = properties;
    }

    public String getUrl() throws BeeException {
        String username = (String) properties.remove(USERNAME);
        String password = (String) properties.remove(PASSWORD);
        String connectionMode = (String) properties.remove(CONNECTION_MODE);
        String sessionMode = (String) properties.remove(SESSION_MODE);
        String datasourceMode = (String) properties.remove(DATASOURCE_MODE);
        String connectionHost = (String) properties.remove(CONNECTION_HOST);
        String connectionPort = (String) properties.remove(CONNECTION_PORT);
        String connectionResource = (String) properties.remove(CONNECTION_RESOURCE);

        if (sessionMode == null || sessionMode.isEmpty()) {
            throw new BeeException(new IllegalArgumentException("must setting 'session_mode'"));
        }

        if (datasourceMode == null || datasourceMode.isEmpty()) {
            throw new BeeException(new IllegalArgumentException("must setting 'datasource_mode'"));
        }

        if (connectionMode == null || connectionMode.isEmpty()) {
            connectionMode = DEFAULT_CONNECT_MODE;
        }

        StringBuilder builder = new StringBuilder();
        builder.append(sessionMode).append(":").append(datasourceMode).append(":").append(connectionMode);
        if (!connectionMode.equals(DEFAULT_CONNECT_MODE)) {
            builder.append("://");
            if (username != null && !username.isEmpty()) {
                builder.append(username).append(":").append(password).append("@");
            }
            if (connectionHost == null || connectionHost.isEmpty()) {
                throw new BeeException(new IllegalArgumentException("must setting 'connection_host'"));
            }
            if (connectionPort == null || connectionPort.isEmpty()) {
                throw new BeeException(new IllegalArgumentException("must setting 'connection_port'"));
            }
            builder.append(connectionHost).append(":").append(connectionPort);
            if (connectionResource != null && !connectionResource.isEmpty()) {
                builder.append("/").append(connectionResource);
            }
            builder.append("?");

            int index = 0;
            Set<Object> keys = properties.keySet();
            for (Object key : keys) {
                String keyStr = key.toString();
                builder.append(keyStr).append("=").append(properties.getProperty(keyStr));
                if (index < keys.size() - 1) {
                    builder.append("&");
                }
                index += 1;
            }
        }
        return builder.toString();
    }

    public int getConnectionTimeout() {
        String connectionTimeout = properties.getProperty(CONNECTION_TIMEOUT);
        return Integer.parseInt(connectionTimeout);
    }

    public void setConnectionTimeout(int timeout) {
        properties.setProperty(CONNECTION_TIMEOUT, "" + timeout);
    }

    public String getApplication() {
        return properties.getProperty(APPLICATION);
    }

    public void setApplication(String application) {
        properties.setProperty(CONNECTION_TIMEOUT, application);
    }

    public String getBeeHost() {
        return beeHost;
    }

    public int getBeePort() {
        return beePort;
    }

    public Properties getProperties() {
        return properties;
    }
}
