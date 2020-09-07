package com.enmotech.nirvana.bee.connector;

import java.sql.SQLException;

public class BeeException extends SQLException {
    static int OTHER_CODE = -1;

    private final int code;
    private final String msg;

    public BeeException(int code, String msg) {
        super(msg, "BEE-" + code, code);
        this.code = code;
        this.msg = msg;
    }

    public BeeException(String msg, Exception e) {
        super(msg, "BEE-" + OTHER_CODE, OTHER_CODE, e);
        if (e instanceof BeeException) {
            BeeException old = (BeeException) e;
            this.code = old.code;
            this.msg = old.msg;
        } else {
            this.code = OTHER_CODE;
            this.msg = e.getMessage();
        }
    }

    public int getCode() {
        return code;
    }

    public String getMsg() {
        return msg;
    }

    @Override
    public String toString() {
        return "BEE-" + code + ":" + msg;
    }
}
