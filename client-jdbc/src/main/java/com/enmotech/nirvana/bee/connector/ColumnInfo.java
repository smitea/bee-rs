package com.enmotech.nirvana.bee.connector;

/**
 * 列名信息
 */
class ColumnInfo {
    private final String name;
    private final DataType type;

    public ColumnInfo(String name, DataType type) {
        this.name = name;
        this.type = type;
    }

    /**
     * 获取列名
     * @return 列名
     */
    public String getName() {
        return name;
    }

    /**
     * 获取该列数据类型
     * @return 数据类型
     */
    public DataType getType() {
        return type;
    }

    @Override
    public String toString() {
        return "ColumnInfo{" +
                "name='" + name + '\'' +
                '}';
    }
}

