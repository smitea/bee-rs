package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.NotSupportException;

import java.sql.ResultSetMetaData;
import java.sql.SQLException;

public class BeeResultMetaData implements ResultSetMetaData {
    private final ColumnInfo[] header;

    public BeeResultMetaData(ColumnInfo[] header) {
        this.header = header;
    }

    @Override
    public int getColumnCount() throws SQLException {
        return header.length;
    }

    @Override
    public boolean isAutoIncrement(int column) throws SQLException {
        return false;
    }

    @Override
    public boolean isCaseSensitive(int column) throws SQLException {
        return false;
    }

    @Override
    public boolean isSearchable(int column) throws SQLException {
        return true;
    }

    @Override
    public boolean isCurrency(int column) throws SQLException {
        return false;
    }

    @Override
    public int isNullable(int column) throws SQLException {
        return ResultSetMetaData.columnNullable;
    }

    @Override
    public boolean isSigned(int column) throws SQLException {
        return false;
    }

    @Override
    public int getColumnDisplaySize(int column) throws SQLException {
        return header[column].getName().length();
    }

    @Override
    public String getColumnLabel(int column) throws SQLException {
        return header[column].getName();
    }

    @Override
    public String getColumnName(int column) throws SQLException {
        return getColumnLabel(column);
    }

    @Override
    public String getSchemaName(int column) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getPrecision(int column) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getScale(int column) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String getTableName(int column) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String getCatalogName(int column) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int getColumnType(int column) throws SQLException {
        return header[column].getType().getType();
    }

    @Override
    public String getColumnTypeName(int column) throws SQLException {
        return header[column].getType().getName();
    }

    @Override
    public boolean isReadOnly(int column) throws SQLException {
        return true;
    }

    @Override
    public boolean isWritable(int column) throws SQLException {
        return false;
    }

    @Override
    public boolean isDefinitelyWritable(int column) throws SQLException {
        return false;
    }

    @Override
    public String getColumnClassName(int column) throws SQLException {
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
