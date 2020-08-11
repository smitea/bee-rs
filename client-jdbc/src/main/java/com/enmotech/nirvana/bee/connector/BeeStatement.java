package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import com.enmotech.nirvana.bee.connector.codec.NotSupportException;
import com.enmotech.nirvana.bee.connector.codec.StatementReq;
import com.enmotech.nirvana.bee.connector.codec.StatementResp;
import com.enmotech.nirvana.bee.connector.codec.PacketHandler;
import io.netty.buffer.ByteBuf;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.SQLWarning;
import java.sql.Statement;
import java.time.Duration;
import java.util.concurrent.TimeUnit;

/**
 * Bee 请求
 */
public class BeeStatement implements Statement{
    private final int id;
    private final BeeConnection connection;
    private final Transport transport;

    private String script;
    private int queryTimeout = 5;

    public BeeStatement(int id, BeeConnection connection, Transport transport) {
        this.id = id;
        this.connection = connection;
        this.transport = transport;
    }

    public BeeStatement(int id, String script, BeeConnection connection, Transport transport) {
        this.id = id;
        this.script = script;
        this.transport = transport;
        this.connection = connection;
    }

    private Response process(int timeout) throws BeeException {
        StatementResp resp = new StatementResp((long) timeout, TimeUnit.SECONDS);
        StatementReq req = new StatementReq(id, script, timeout);
        ResponsePacketHandler handler = new ResponsePacketHandler(resp, req);
        transport.writePacket(req, handler);
        return resp;
    }

    @Override
    public ResultSet executeQuery(String sql) throws SQLException {
        this.script = sql;
        return new BeeResultSet(this, process(queryTimeout));
    }

    @Override
    public int executeUpdate(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void close() throws SQLException {

    }

    @Override
    public int getMaxFieldSize() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setMaxFieldSize(int max) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getMaxRows() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setMaxRows(int max) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setEscapeProcessing(boolean enable) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getQueryTimeout() throws SQLException {
        return queryTimeout;
    }

    @Override
    public void setQueryTimeout(int seconds) throws SQLException {
        this.queryTimeout = seconds;
    }

    @Override
    public void cancel() throws SQLException {
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
    public void setCursorName(String name) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean execute(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public ResultSet getResultSet() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getUpdateCount() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean getMoreResults() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setFetchDirection(int direction) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getFetchDirection() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setFetchSize(int rows) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getFetchSize() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getResultSetConcurrency() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getResultSetType() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void addBatch(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void clearBatch() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int[] executeBatch() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Connection getConnection() throws SQLException {
        return this.connection;
    }

    @Override
    public boolean getMoreResults(int current) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public ResultSet getGeneratedKeys() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int executeUpdate(String sql, int autoGeneratedKeys) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int executeUpdate(String sql, int[] columnIndexes) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int executeUpdate(String sql, String[] columnNames) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean execute(String sql, int autoGeneratedKeys) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean execute(String sql, int[] columnIndexes) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean execute(String sql, String[] columnNames) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getResultSetHoldability() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isClosed() throws SQLException {
        return transport.isClosed();
    }

    @Override
    public void setPoolable(boolean poolable) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isPoolable() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void closeOnCompletion() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isCloseOnCompletion() throws SQLException {
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

    static class ResponsePacketHandler implements PacketHandler {
        private final StatementResp resp;
        private final StatementReq req;

        public ResponsePacketHandler(StatementResp resp, StatementReq req) {
            this.resp = resp;
            this.req = req;
        }

        @Override
        public void decode(ByteBuf packet) {
            resp.decode(packet);
        }

        @Override
        public boolean validPacket(ByteBuf packet) {
            try {
                StatementResp resp = new StatementResp();
                resp.decode(packet);
                return req.getId() == resp.getId();
            } catch (Exception e) {
                return false;
            }
        }

        @Override
        public int type() {
            return resp.type();
        }

        @Override
        public boolean isMulti() {
            return true;
        }

        @Override
        public boolean isEnd() {
            try {
                return resp.isAbort();
            } catch (Exception e) {
                return false;
            }
        }
    }
}
