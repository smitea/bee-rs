package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.Bytes;

/**
 * Bee 值类型
 */
public class Value {
    private final DataType type;
    private final Object value;

    private Value(DataType type, Object value) {
        this.type = type;
        this.value = value;
    }

    public DataType getType() {
        return type;
    }

    public Object getValue() {
        return value;
    }

    @Override
    public String toString() {
        if (value != null) {
            if (type == DataType.STRING) {
                return "\"" + value + "\"";
            }
            return value.toString();
        } else {
            return "nil";
        }
    }

    /**
     * 转换 String 类型值
     *
     * @param value value
     * @return Value
     */
    public static Value str(String value) {
        return new Value(DataType.STRING, value);
    }

    /**
     * 转换 String 类型值
     *
     * @param value value
     * @return Value
     */
    public static Value number(Double value) {
        return new Value(DataType.NUMBER, value);
    }

    /**
     * 转换 Integer 类型值
     *
     * @param value value
     * @return Value
     */
    public static Value integer(long value) {
        return new Value(DataType.INTEGER, value);
    }

    /**
     * 转换 Integer 类型值
     *
     * @param value value
     * @return Value
     */
    public static Value integer(int value) {
        return new Value(DataType.INTEGER, value);
    }

    /**
     * 转换 Boolean 类型值
     *
     * @param value value
     * @return Value
     */
    public static Value bool(Boolean value) {
        return new Value(DataType.BOOLEAN, value);
    }

    /**
     * 转换 Nil 类型值
     *
     * @return Value
     */
    public static Value nil() {
        return new Value(DataType.NIL, null);
    }

    /**
     * 转换 Bytes 类型值
     *
     * @return Value
     */
    public static Value bytes(Bytes bytes) {
        return new Value(DataType.NIL, bytes);
    }
}
