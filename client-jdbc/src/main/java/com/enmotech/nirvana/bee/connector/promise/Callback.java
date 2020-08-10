package com.enmotech.nirvana.bee.connector.promise;

/**
 * Promise回调接口
 * @author smitea
 */
public interface Callback<T> {
    void onSuccess(T value);
    void onFailure(Throwable value);
}