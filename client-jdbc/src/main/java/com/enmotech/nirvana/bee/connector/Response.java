package com.enmotech.nirvana.bee.connector;

interface Response {
    ColumnInfo[] getColumns() throws BeeException;

    ResultRow next() throws BeeException;

    boolean hasNext() throws BeeException;
}
