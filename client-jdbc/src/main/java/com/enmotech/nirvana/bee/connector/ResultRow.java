package com.enmotech.nirvana.bee.connector;

class ResultRow {
    private final ColumnInfo[] header;
    private final Value[] values;

    public ResultRow(final ColumnInfo[] header, final Value[] values) {
        this.header = header;
        this.values = values;
    }

    @Override
    public String toString() {
        final StringBuilder builder = new StringBuilder();
        builder.append("{");
        for (int i = 0; i < header.length; i++) {
            final Value value = values[i];
            builder.append("\"").append(header[i].getName()).append("\" : ").append(value).append(",");
        }
        builder.append("}");
        return builder.toString();
    }

    public Double getNumber(final String name) {
        return findValue(name);
    }

    public String getString(final String name) {
        return findValue(name);
    }

    public Long getLong(final String name) {
        return findValue(name);
    }

    public Boolean getBoolean(final String name) {
        return findValue(name);
    }

    public Bytes getBytes(final String name) {
        return findValue(name);
    }

    public Object getObject(final String name) {
        return findValue(name);
    }

    public boolean isNil(final String name) {
        int index = findIndex(name);
        if (index != -1) {
            return values[index].getType() == DataType.NIL;
        }
        return true;
    }

    public Double getNumber(final int index) {
        return findValue(index);
    }

    public String getString(final int index) {
        return findValue(index);
    }

    public Long getLong(final int index) {
        return findValue(index);
    }

    public Boolean getBoolean(final int index) {
        return findValue(index);
    }

    public Bytes getBytes(final int index) {
        return findValue(index);
    }

    public Object getObject(final int index) {
        return findValue(index);
    }

    public boolean isNil(final int index) {
        return findValue(index) == null;
    }

    public boolean isEmpty() {
        return values.length == 0;
    }

    private <T> T findValue(final String name) {
        int index = findIndex(name);
        if (index != -1) {
            return findValue(index);
        }
        return null;
    }

    private <T> T findValue(int index) {
        //noinspection unchecked
        return (T) values[index].getValue();
    }

    public int findIndex(final String name) {
        for (int i = 0; i < header.length; i++) {
            if (name.equals(header[i].getName())) {
                return i;
            }
        }
        return -1;
    }
}
