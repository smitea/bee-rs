package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import com.enmotech.nirvana.bee.connector.codec.ConnectReq;
import com.enmotech.nirvana.bee.connector.codec.ConnectResp;
import com.enmotech.nirvana.bee.connector.codec.NotConnectedException;
import com.enmotech.nirvana.bee.connector.codec.NotSupportException;
import java.io.Closeable;
import java.sql.Array;
import java.sql.Blob;
import java.sql.CallableStatement;
import java.sql.Clob;
import java.sql.Connection;
import java.sql.DatabaseMetaData;
import java.sql.NClob;
import java.sql.PreparedStatement;
import java.sql.SQLClientInfoException;
import java.sql.SQLException;
import java.sql.SQLWarning;
import java.sql.SQLXML;
import java.sql.Savepoint;
import java.sql.Statement;
import java.sql.Struct;
import java.util.Map;
import java.util.Properties;
import java.util.concurrent.Executor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

class BeeConnection implements Connection, Closeable {
    static int MAX_STATEMENT_NUM = 65535;

    private final Transport transport;
    private final AtomicInteger id;
    private final ClientInfo clientInfo;

    public BeeConnection(ClientInfo clientInfo) throws BeeException {
        this.transport = createTransport(clientInfo);
        this.id = new AtomicInteger(0);
        this.clientInfo = clientInfo;
    }

    private Transport createTransport(ClientInfo clientInfo) throws BeeException {
        try {
            Transport transport = new Transport(clientInfo.getBeeHost(), clientInfo.getBeePort(),
                    clientInfo.getConnectionTimeout());
            ConnectResp resp = transport
                    .writePacket(new ConnectReq(clientInfo.getUrl(), clientInfo.getApplication()), ConnectResp.class)
                    .await(clientInfo.getConnectionTimeout() + 1, TimeUnit.SECONDS);

            if (!resp.isOk()) {
                throw resp.getException();
            }
            return transport;
        } catch (Exception e) {
            throw new BeeException(e);
        }
    }

    private int getClientId() {
        int statementId = id.addAndGet(1);
        if (statementId >= MAX_STATEMENT_NUM) {
            statementId = 0;
            id.set(statementId);
        }
        return statementId;
    }

    @Override
    public Statement createStatement() throws SQLException {
        if (transport.isClosed()) {
            throw new NotConnectedException();
        }
        return new BeeStatement(getClientId(), this, transport);
    }

    @Override
    public PreparedStatement prepareStatement(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public CallableStatement prepareCall(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String nativeSQL(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setAutoCommit(boolean autoCommit) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean getAutoCommit() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void commit() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void rollback() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void close() {
        this.transport.close();
    }

    @Override
    public boolean isClosed() {
        return transport.isClosed();
    }

    @Override
    public DatabaseMetaData getMetaData() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setReadOnly(boolean readOnly) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isReadOnly() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setCatalog(String catalog) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String getCatalog() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setTransactionIsolation(int level) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getTransactionIsolation() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public SQLWarning getWarnings() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void clearWarnings() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Statement createStatement(int resultSetType, int resultSetConcurrency) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int resultSetType, int resultSetConcurrency)
            throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public CallableStatement prepareCall(String sql, int resultSetType, int resultSetConcurrency) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Map<String, Class<?>> getTypeMap() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setTypeMap(Map<String, Class<?>> map) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setHoldability(int holdability) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getHoldability() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Savepoint setSavepoint() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Savepoint setSavepoint(String name) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void rollback(Savepoint savepoint) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void releaseSavepoint(Savepoint savepoint) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Statement createStatement(int resultSetType, int resultSetConcurrency, int resultSetHoldability)
            throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int resultSetType, int resultSetConcurrency,
            int resultSetHoldability) throws SQLException {
        return prepareStatement(sql);
    }

    @Override
    public CallableStatement prepareCall(String sql, int resultSetType, int resultSetConcurrency,
            int resultSetHoldability) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int autoGeneratedKeys) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int[] columnIndexes) throws SQLException {
        return prepareStatement(sql);
    }

    @Override
    public PreparedStatement prepareStatement(String sql, String[] columnNames) throws SQLException {
        return prepareStatement(sql);
    }

    @Override
    public Clob createClob() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Blob createBlob() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public NClob createNClob() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public SQLXML createSQLXML() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isValid(int timeout) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setClientInfo(String name, String value) throws SQLClientInfoException {
        clientInfo.getProperties().setProperty(name, value);
    }

    @Override
    public void setClientInfo(Properties properties) throws SQLClientInfoException {
        Properties old = this.clientInfo.getProperties();
        properties.keySet().stream().forEach(key -> {
            String keyStr = (String) key;
            String value = properties.getProperty(keyStr);
            old.setProperty(keyStr, value);
        });
    }

    @Override
    public String getClientInfo(String name) throws SQLException {
        return clientInfo.getProperties().getProperty(name);
    }

    @Override
    public Properties getClientInfo() throws SQLException {
        return clientInfo.getProperties();
    }

    @Override
    public Array createArrayOf(String typeName, Object[] elements) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Struct createStruct(String typeName, Object[] attributes) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setSchema(String schema) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String getSchema() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void abort(Executor executor) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setNetworkTimeout(Executor executor, int milliseconds) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getNetworkTimeout() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public <T> T unwrap(Class<T> iface) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isWrapperFor(Class<?> iface) throws SQLException {
        throw new NotSupportException();
    }
}