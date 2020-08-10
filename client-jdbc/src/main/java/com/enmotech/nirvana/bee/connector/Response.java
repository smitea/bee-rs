package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;

import java.sql.ResultSet;

public interface Response {
    ColumnInfo[] getColumns() throws BeeException;

    ResultRow next() throws BeeException;

    boolean hasNext() throws BeeException;
}
