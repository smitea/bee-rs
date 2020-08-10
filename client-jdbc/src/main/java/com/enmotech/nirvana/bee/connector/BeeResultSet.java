package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import com.enmotech.nirvana.bee.connector.codec.Bytes;
import com.enmotech.nirvana.bee.connector.codec.NotSupportException;

import java.io.IOException;
import java.io.InputStream;
import java.io.Reader;
import java.io.StringReader;
import java.math.BigDecimal;
import java.net.URL;
import java.sql.Array;
import java.sql.Blob;
import java.sql.Clob;
import java.sql.Date;
import java.sql.NClob;
import java.sql.Ref;
import java.sql.ResultSet;
import java.sql.ResultSetMetaData;
import java.sql.RowId;
import java.sql.SQLException;
import java.sql.SQLWarning;
import java.sql.SQLXML;
import java.sql.Statement;
import java.sql.Time;
import java.sql.Timestamp;
import java.util.Calendar;
import java.util.Map;

public class BeeResultSet implements ResultSet {
    private final BeeStatement statement;
    private final Response response;
    private ResultRow row;
    private int index;

    public BeeResultSet(BeeStatement statement, Response response) {
        this.response = response;
        this.statement = statement;
        this.index = 0;
    }

    @Override
    public boolean next() throws SQLException {
        if (response.hasNext()) {
            row = response.next();
            index += 1;
        }
        return false;
    }

    @Override
    public void close() throws SQLException {
    }

    @Override
    public boolean wasNull() throws SQLException {
        return row.isEmpty();
    }

    @Override
    public String getString(int columnIndex) throws SQLException {
        return row.getString(columnIndex);
    }

    @Override
    public boolean getBoolean(int columnIndex) throws SQLException {
        return row.getBoolean(columnIndex);
    }

    @Override
    public byte getByte(int columnIndex) throws SQLException {
        return row.getLong(columnIndex).byteValue();
    }

    @Override
    public short getShort(int columnIndex) throws SQLException {
        return row.getLong(columnIndex).shortValue();
    }

    @Override
    public int getInt(int columnIndex) throws SQLException {
        return row.getLong(columnIndex).intValue();
    }

    @Override
    public long getLong(int columnIndex) throws SQLException {
        return row.getLong(columnIndex);
    }

    @Override
    public float getFloat(int columnIndex) throws SQLException {
        return row.getNumber(columnIndex).floatValue();
    }

    @Override
    public double getDouble(int columnIndex) throws SQLException {
        return row.getNumber(columnIndex);
    }

    @Override
    public BigDecimal getBigDecimal(int columnIndex, int scale) throws SQLException {
        return BigDecimal.valueOf(row.getNumber(columnIndex));
    }

    @Override
    public byte[] getBytes(int columnIndex) throws SQLException {
        Bytes bytes = row.getBytes(columnIndex);
        try {
            return bytes.toBytes();
        } catch (IOException e) {
            throw new BeeException(e);
        }
    }

    @Override
    public Date getDate(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Time getTime(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Timestamp getTimestamp(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public InputStream getAsciiStream(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public InputStream getUnicodeStream(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public InputStream getBinaryStream(int columnIndex) throws SQLException {
        return row.getBytes(columnIndex).getBinaryStream();
    }

    @Override
    public String getString(String columnLabel) throws SQLException {
        return row.getString(columnLabel);
    }

    @Override
    public boolean getBoolean(String columnLabel) throws SQLException {
        return row.getBoolean(columnLabel);
    }

    @Override
    public byte getByte(String columnLabel) throws SQLException {
        return row.getLong(columnLabel).byteValue();
    }

    @Override
    public short getShort(String columnLabel) throws SQLException {
        return row.getLong(columnLabel).byteValue();
    }

    @Override
    public int getInt(String columnLabel) throws SQLException {
        return row.getLong(columnLabel).intValue();
    }

    @Override
    public long getLong(String columnLabel) throws SQLException {
        return row.getLong(columnLabel);
    }

    @Override
    public float getFloat(String columnLabel) throws SQLException {
        return row.getNumber(columnLabel).floatValue();
    }

    @Override
    public double getDouble(String columnLabel) throws SQLException {
        return row.getNumber(columnLabel);
    }

    @Override
    public BigDecimal getBigDecimal(String columnLabel, int scale) throws SQLException {
        return BigDecimal.valueOf(row.getNumber(columnLabel));
    }

    @Override
    public byte[] getBytes(String columnLabel) throws SQLException {
        try {
            return row.getBytes(columnLabel).toBytes();
        } catch (IOException e) {
            throw new BeeException(e);
        }
    }

    @Override
    public Date getDate(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Time getTime(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Timestamp getTimestamp(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public InputStream getAsciiStream(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public InputStream getUnicodeStream(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public InputStream getBinaryStream(String columnLabel) throws SQLException {
        return row.getBytes(columnLabel).getBinaryStream();
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
    public String getCursorName() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public ResultSetMetaData getMetaData() throws SQLException {
        return new BeeResultMetaData(response.getColumns());
    }

    @Override
    public Object getObject(int columnIndex) throws SQLException {
        return row.getObject(columnIndex);
    }

    @Override
    public Object getObject(String columnLabel) throws SQLException {
        return row.getObject(columnLabel);
    }

    @Override
    public int findColumn(String columnLabel) throws SQLException {
        return row.findIndex(columnLabel);
    }

    @Override
    public Reader getCharacterStream(int columnIndex) throws SQLException {
        return new StringReader(row.getString(columnIndex));
    }

    @Override
    public Reader getCharacterStream(String columnLabel) throws SQLException {
        return new StringReader(row.getString(columnLabel));
    }

    @Override
    public BigDecimal getBigDecimal(int columnIndex) throws SQLException {
        return BigDecimal.valueOf(row.getNumber(columnIndex));
    }

    @Override
    public BigDecimal getBigDecimal(String columnLabel) throws SQLException {
        return BigDecimal.valueOf(row.getNumber(columnLabel));
    }

    @Override
    public boolean isBeforeFirst() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isAfterLast() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isFirst() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isLast() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void beforeFirst() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void afterLast() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean first() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean last() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getRow() throws SQLException {
        return index;
    }

    @Override
    public boolean absolute(int row) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean relative(int rows) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean previous() throws SQLException {
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
    public int getType() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getConcurrency() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean rowUpdated() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean rowInserted() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean rowDeleted() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNull(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBoolean(int columnIndex, boolean x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateByte(int columnIndex, byte x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateShort(int columnIndex, short x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateInt(int columnIndex, int x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateLong(int columnIndex, long x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateFloat(int columnIndex, float x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateDouble(int columnIndex, double x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBigDecimal(int columnIndex, BigDecimal x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateString(int columnIndex, String x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBytes(int columnIndex, byte[] x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateDate(int columnIndex, Date x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateTime(int columnIndex, Time x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateTimestamp(int columnIndex, Timestamp x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateAsciiStream(int columnIndex, InputStream x, int length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBinaryStream(int columnIndex, InputStream x, int length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateCharacterStream(int columnIndex, Reader x, int length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateObject(int columnIndex, Object x, int scaleOrLength) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateObject(int columnIndex, Object x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNull(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBoolean(String columnLabel, boolean x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateByte(String columnLabel, byte x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateShort(String columnLabel, short x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateInt(String columnLabel, int x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateLong(String columnLabel, long x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateFloat(String columnLabel, float x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateDouble(String columnLabel, double x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBigDecimal(String columnLabel, BigDecimal x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateString(String columnLabel, String x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBytes(String columnLabel, byte[] x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateDate(String columnLabel, Date x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateTime(String columnLabel, Time x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateTimestamp(String columnLabel, Timestamp x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateAsciiStream(String columnLabel, InputStream x, int length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBinaryStream(String columnLabel, InputStream x, int length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateCharacterStream(String columnLabel, Reader reader, int length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateObject(String columnLabel, Object x, int scaleOrLength) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateObject(String columnLabel, Object x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void insertRow() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateRow() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void deleteRow() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void refreshRow() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void cancelRowUpdates() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void moveToInsertRow() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void moveToCurrentRow() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Statement getStatement() throws SQLException {
        return statement;
    }

    @Override
    public Object getObject(int columnIndex, Map<String, Class<?>> map) throws SQLException {
        return getObject(columnIndex);
    }

    @Override
    public Ref getRef(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Blob getBlob(int columnIndex) throws SQLException {
        return row.getBytes(columnIndex);
    }

    @Override
    public Clob getClob(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Array getArray(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Object getObject(String columnLabel, Map<String, Class<?>> map) throws SQLException {
        return row.getObject(columnLabel);
    }

    @Override
    public Ref getRef(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Blob getBlob(String columnLabel) throws SQLException {
        return row.getBytes(columnLabel);
    }

    @Override
    public Clob getClob(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Array getArray(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Date getDate(int columnIndex, Calendar cal) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Date getDate(String columnLabel, Calendar cal) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Time getTime(int columnIndex, Calendar cal) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Time getTime(String columnLabel, Calendar cal) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Timestamp getTimestamp(int columnIndex, Calendar cal) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Timestamp getTimestamp(String columnLabel, Calendar cal) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public URL getURL(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public URL getURL(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateRef(int columnIndex, Ref x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateRef(String columnLabel, Ref x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBlob(int columnIndex, Blob x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBlob(String columnLabel, Blob x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateClob(int columnIndex, Clob x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateClob(String columnLabel, Clob x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateArray(int columnIndex, Array x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateArray(String columnLabel, Array x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public RowId getRowId(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public RowId getRowId(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateRowId(int columnIndex, RowId x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateRowId(String columnLabel, RowId x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getHoldability() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isClosed() throws SQLException {
        return statement.isClosed();
    }

    @Override
    public void updateNString(int columnIndex, String nString) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNString(String columnLabel, String nString) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNClob(int columnIndex, NClob nClob) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNClob(String columnLabel, NClob nClob) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public NClob getNClob(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public NClob getNClob(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public SQLXML getSQLXML(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public SQLXML getSQLXML(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateSQLXML(int columnIndex, SQLXML xmlObject) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateSQLXML(String columnLabel, SQLXML xmlObject) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String getNString(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String getNString(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Reader getNCharacterStream(int columnIndex) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Reader getNCharacterStream(String columnLabel) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNCharacterStream(int columnIndex, Reader x, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNCharacterStream(String columnLabel, Reader reader, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateAsciiStream(int columnIndex, InputStream x, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBinaryStream(int columnIndex, InputStream x, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateCharacterStream(int columnIndex, Reader x, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateAsciiStream(String columnLabel, InputStream x, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBinaryStream(String columnLabel, InputStream x, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateCharacterStream(String columnLabel, Reader reader, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBlob(int columnIndex, InputStream inputStream, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBlob(String columnLabel, InputStream inputStream, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateClob(int columnIndex, Reader reader, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateClob(String columnLabel, Reader reader, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNClob(int columnIndex, Reader reader, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNClob(String columnLabel, Reader reader, long length) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNCharacterStream(int columnIndex, Reader x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNCharacterStream(String columnLabel, Reader reader) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateAsciiStream(int columnIndex, InputStream x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBinaryStream(int columnIndex, InputStream x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateCharacterStream(int columnIndex, Reader x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateAsciiStream(String columnLabel, InputStream x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBinaryStream(String columnLabel, InputStream x) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateCharacterStream(String columnLabel, Reader reader) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBlob(int columnIndex, InputStream inputStream) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateBlob(String columnLabel, InputStream inputStream) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateClob(int columnIndex, Reader reader) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateClob(String columnLabel, Reader reader) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNClob(int columnIndex, Reader reader) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void updateNClob(String columnLabel, Reader reader) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public <T> T getObject(int columnIndex, Class<T> type) throws SQLException {
        //noinspection unchecked
        return (T) row.getObject(columnIndex);
    }

    @Override
    public <T> T getObject(String columnLabel, Class<T> type) throws SQLException {
        //noinspection unchecked
        return (T) row.getObject(columnLabel);
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
