package com.enmotech.nirvana.bee.connector;

public enum DataType {
    NIL("Null",0x00),
    STRING("String",0x01),
    INTEGER("Integer",0x02),
    NUMBER("Number",0x03),
    BOOLEAN("Boolean",0x04),
    BYTES("Bytes",0x05);

    private final String name;
    private final Integer type;

    DataType(String name, Integer type) {
        this.name = name;
        this.type = type;
    }

    public static DataType valueOf(int code){
        for (DataType item: values()){
            if (item.type == code){
                return item;
            }
        }
        return DataType.NIL;
    }

    public String getName() {
        return name;
    }

    public Integer getType() {
        return type;
    }
}
