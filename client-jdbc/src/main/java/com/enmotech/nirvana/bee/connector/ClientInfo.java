package com.enmotech.nirvana.bee.connector;

import java.net.InetAddress;
import java.net.MalformedURLException;
import java.net.URI;
import java.net.URL;
import java.net.UnknownHostException;
import java.sql.DriverPropertyInfo;
import java.sql.SQLException;
import java.util.Properties;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

class ClientInfo {
    static String USERNAME = "username";
    static String PASSWORD = "password";
    static String CONNECTION_MODE = "connect_mode";
    static String SESSION_MODE = "session_mode";
    static String DATASOURCE_MODE = "datasource_mode";
    static String PROXY_HOST = "proxy_host";
    static String PROXY_PORT = "proxy_port";
    static String BEE_HOST = "bee_host";
    static String BEE_PORT = "bee_port";
    static String CONNECTION_RESOURCE = "connection_resource";
    static String CONNECTION_TIMEOUT = "connection_timeout";
    static String SOCKET_TIMEOUT = "socket_timeout";
    static String APPLICATION = "application";
    static String DEFAULT_CONNECT_MODE = "default";
    static String OS_VERSION = "os_version";
    static String ENVIRONMENTS = "environments";
    static String URL_PATTERN = "(\\S+):(\\S+):(\\S+://\\S+)";
    static final Properties DEFAULT_PROPERTIES;

    static {
        DEFAULT_PROPERTIES = new Properties();
        DEFAULT_PROPERTIES.setProperty(ClientInfo.CONNECTION_TIMEOUT, "10");
        DEFAULT_PROPERTIES.setProperty(ClientInfo.BEE_HOST, "127.0.0.1");
        DEFAULT_PROPERTIES.setProperty(ClientInfo.BEE_PORT, "6142");
        DEFAULT_PROPERTIES.setProperty(ClientInfo.SOCKET_TIMEOUT, "20");
        DEFAULT_PROPERTIES.setProperty(ClientInfo.APPLICATION, "jdbc");
        DEFAULT_PROPERTIES.setProperty(ClientInfo.OS_VERSION, "Unknown");
        DEFAULT_PROPERTIES.setProperty(ClientInfo.ENVIRONMENTS, "[BETHUNE_PATH: bee]");
    }

    private final String beeHost;
    private final int beePort;

    protected int socketTimeout;
    protected String application;

    protected final Properties properties;
    protected final Properties environments;

    public ClientInfo(String beeHost, int beePort) {
        this.beeHost = beeHost;
        this.beePort = beePort;
        this.properties = DEFAULT_PROPERTIES;
        this.environments = new Properties();
        initApplication();
    }

    public ClientInfo(String url, Properties properties) throws SQLException {
        this.properties = properties.isEmpty() ? DEFAULT_PROPERTIES : properties;
        this.environments = new Properties();

        Pattern pattern = Pattern.compile(URL_PATTERN);
        Matcher matcher = pattern.matcher(url);
        if (matcher.find()) {
            String sessMode = matcher.group(1);
            String dsMode = matcher.group(2);
            String connectUrl = matcher.group(3);

            try {
                URI connectUrlPath = URI.create(connectUrl);
                String connectionMode = connectUrlPath.getScheme();
                String userInfo = connectUrlPath.getUserInfo();

                if (userInfo != null) {
                    String[] userInfos = userInfo.split(":");
                    if (userInfos.length > 1) {
                        String username = userInfos[0];
                        String password = userInfos[1];
                        this.properties.setProperty(USERNAME, username);
                        this.properties.setProperty(PASSWORD, password);
                    } else if (userInfos.length == 1) {
                        this.properties.setProperty(USERNAME, userInfo);
                    }
                }
                String proxyHost = connectUrlPath.getHost();
                int proxyPort = connectUrlPath.getPort();

                String query = connectUrlPath.getQuery();
                String resource = connectUrlPath.getPath().replaceFirst("/", "");
                if (query != null) {
                    String[] params = query.split("&");

                    for (String param : params) {
                        String[] paramItem = param.split("=");
                        String paramName = paramItem[0];
                        String paramValue = paramItem[1];
                        this.properties.setProperty(paramName, paramValue);
                    }
                }

                this.beeHost = this.properties.getProperty(BEE_HOST, "127.0.0.1");
                this.beePort = Integer.parseInt(this.properties.getProperty(BEE_PORT, "6142"));
                this.application = this.properties.getProperty(APPLICATION, "");
                if (application.isEmpty()) {
                    initApplication();
                }
                this.socketTimeout = Integer.parseInt(this.properties.getProperty(SOCKET_TIMEOUT, "10000"));
                this.properties.setProperty(PROXY_HOST, proxyHost);
                this.properties.setProperty(PROXY_PORT, "" + proxyPort);
                this.properties.setProperty(CONNECTION_RESOURCE, resource);
                this.properties.setProperty(DATASOURCE_MODE, dsMode);
                this.properties.setProperty(SESSION_MODE, sessMode);
                this.properties.setProperty(CONNECTION_MODE, connectionMode);
            } catch (Exception e) {
                String msg = e.getMessage();
                if (msg == null) {
                    msg = e.getLocalizedMessage();
                }
                throw new BeeException(-1, msg);
            }
        } else {
            throw new BeeException(-1, "Can't match url");
        }
    }

    public void initApplication() {
        try {
            application = InetAddress.getLocalHost().getHostName();
        } catch (UnknownHostException e) {
            application = "jdbc";
        }
    }

    public String getUrl() throws BeeException {
        String username = (String) properties.remove(USERNAME);
        String password = (String) properties.remove(PASSWORD);
        String connectionMode = (String) properties.remove(CONNECTION_MODE);
        String sessionMode = (String) properties.remove(SESSION_MODE);
        String datasourceMode = (String) properties.remove(DATASOURCE_MODE);
        String proxyHost = (String) properties.remove(PROXY_HOST);
        String proxyPort = (String) properties.remove(PROXY_PORT);
        String connectionResource = (String) properties.remove(CONNECTION_RESOURCE);

        if (sessionMode == null || sessionMode.isEmpty()) {
            throw new BeeException("must setting 'session_mode'", new IllegalArgumentException());
        }

        if (datasourceMode == null || datasourceMode.isEmpty()) {
            throw new BeeException("must setting 'datasource_mode'", new IllegalArgumentException());
        }

        if (connectionMode == null || connectionMode.isEmpty()) {
            connectionMode = DEFAULT_CONNECT_MODE;
        }

        StringBuilder builder = new StringBuilder();
        builder.append(sessionMode).append(":").append(datasourceMode).append(":").append(connectionMode);
        if (!connectionMode.equals(DEFAULT_CONNECT_MODE)) {
            builder.append("://");
            if (username != null && !username.isEmpty()) {
                builder.append(username).append(":");
                if (password != null) {
                    builder.append(password);
                }
                builder.append("@");
            }
            if (proxyHost == null || proxyHost.isEmpty()) {
                throw new BeeException("must setting 'connection_host'", new IllegalArgumentException());
            }
            if (proxyPort == null || proxyPort.isEmpty()) {
                throw new BeeException("must setting 'connection_port'", new IllegalArgumentException());
            }
            builder.append(proxyHost).append(":").append(proxyPort);
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

            builder.append("&");
            builder.append(ENVIRONMENTS);
            builder.append("=");
            builder.append("[");
            Set<Object> envs = environments.keySet();
            for (Object env : envs) {
                String keyStr = env.toString();
                builder.append(keyStr).append(":").append(environments.getProperty(keyStr));
                if (index < keys.size() - 1) {
                    builder.append(",");
                }
                index += 1;
            }
            builder.append("]");
        }
        return builder.toString();
    }

    public int getConnectionTimeout() {
        String connectionTimeout = properties.getProperty(CONNECTION_TIMEOUT);
        if (connectionTimeout == null) {
            connectionTimeout = "10";
        }
        return Integer.parseInt(connectionTimeout);
    }

    public String getUsername() {
        return properties.getProperty(USERNAME, "");
    }

    public String getResource() {
        return properties.getProperty(CONNECTION_RESOURCE, "");
    }

    public String getApplication() {
        return application;
    }

    public String getHost() {
        return beeHost;
    }

    public int getPort() {
        return beePort;
    }

    public int getSocketTimeout() {
        return socketTimeout;
    }

    public void setSocketTimeout(int socketTimeout) {
        this.socketTimeout = socketTimeout;
    }

    public Properties getProperties() {
        return properties;
    }
}
