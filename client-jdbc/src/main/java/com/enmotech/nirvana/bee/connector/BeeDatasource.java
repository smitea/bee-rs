package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import com.enmotech.nirvana.bee.connector.codec.NotSupportException;

import javax.sql.DataSource;
import java.io.PrintWriter;
import java.sql.Connection;
import java.sql.SQLException;
import java.sql.SQLFeatureNotSupportedException;
import java.util.logging.Logger;

public abstract class BeeDatasource implements DataSource {
    protected ClientInfo clientInfo;

    private String username;
    private String password;

    private int loginTimeout;
    private int socketTimeout;
    private int connectTimeout;

    private String connectionMode = "default";
    private String dataSourceMode = "agent";
    private SessionMode sessionMode = SessionMode.SQLITE;

    private String version;
    private String resource;
    private String application;

    private String proxyHost;
    private int proxyPort;

    BeeDatasource(String host, int port) {
        clientInfo = new ClientInfo(host, port);
    }

    @Override
    public Connection getConnection() throws SQLException {
        return new BeeConnection(clientInfo);
    }

    @Override
    public Connection getConnection(String username, String password) throws SQLException {
        setUsername(username);
        setPassword(password);
        return new BeeConnection(clientInfo);
    }

    @Override
    public <T> T unwrap(Class<T> iface) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isWrapperFor(Class<?> iface) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PrintWriter getLogWriter() throws SQLException {
        return null;
    }

    @Override
    public void setLogWriter(PrintWriter out) throws SQLException {

    }

    @Override
    public Logger getParentLogger() throws SQLFeatureNotSupportedException {
        return null;
    }

    @Override
    public void setLoginTimeout(int seconds) throws SQLException {
        loginTimeout = seconds;
    }

    @Override
    public int getLoginTimeout() throws SQLException {
        return loginTimeout;
    }

    public String getProxyHost() {
        return proxyHost;
    }

    protected void setProxyHost(String proxyHost) {
        clientInfo.properties.setProperty(ClientInfo.PROXY_HOST, proxyHost);
        this.proxyHost = proxyHost;
    }

    public int getProxyPort() {
        return proxyPort;
    }

    void setProxyPort(int proxyPort) {
        clientInfo.properties.setProperty(ClientInfo.PROXY_PORT, "" + proxyPort);
        this.proxyPort = proxyPort;
    }

    public String getUsername() {
        return username;
    }

    protected void setUsername(String username) {
        clientInfo.properties.setProperty(ClientInfo.USERNAME, username);
        this.username = username;
    }

    public String getPassword() {
        return password;
    }

    protected void setPassword(String password) {
        clientInfo.properties.setProperty(ClientInfo.PASSWORD, password);
        this.password = password;
    }

    public int getSocketTimeout() {
        return socketTimeout;
    }

    public void setSocketTimeout(int socketTimeout) {
        clientInfo.socketTimeout = socketTimeout;
        this.socketTimeout = socketTimeout;
    }

    public int getConnectTimeout() {
        return connectTimeout;
    }

    public void setConnectTimeout(int connectTimeout) {
        this.connectTimeout = connectTimeout;
    }

    public String getConnectionMode() {
        return connectionMode;
    }

    protected void setConnectionMode(String connectionMode) {
        clientInfo.properties.setProperty(ClientInfo.CONNECTION_MODE, connectionMode);
        this.connectionMode = connectionMode;
    }

    public String getDataSourceMode() {
        return dataSourceMode;
    }

    protected void setDataSourceMode(String dataSourceMode) {
        clientInfo.properties.setProperty(ClientInfo.DATASOURCE_MODE, dataSourceMode);
        this.dataSourceMode = dataSourceMode;
    }

    public SessionMode getSessionMode() {
        return sessionMode;
    }

    public void setSessionMode(SessionMode sessionMode) {
        clientInfo.properties.setProperty(ClientInfo.SESSION_MODE, sessionMode.getMode());
        this.sessionMode = sessionMode;
    }

    public String getVersion() {
        return version;
    }

    public void setVersion(String version) {
        clientInfo.properties.setProperty(ClientInfo.OS_VERSION, version);
        this.version = version;
    }

    public String getResource() {
        return resource;
    }

    public void setResource(String resource) {
        clientInfo.properties.setProperty(ClientInfo.CONNECTION_RESOURCE, resource);
        this.resource = resource;
    }

    public String getApplication() {
        return application;
    }

    public void setApplication(String application) {
        clientInfo.application = application;
        this.application = application;
    }

    public void setEnv(String key, String value) throws SQLException {
        if ("PATH".equals(key)) {
            throw new BeeException(-1, "Don't set 'PATH' to environments");
        }
        clientInfo.environments.setProperty(key, value);
    }

    public enum SessionMode {
        LUA("lua"),
        SQLITE("sqlite");

        private final String mode;

        SessionMode(String mode) {
            this.mode = mode;
        }

        public String getMode() {
            return mode;
        }
    }
}
