package com.enmotech.nirvana.bee.connector.promise;

import java.util.concurrent.TimeUnit;

/**
 * 同步Future接口
 * @author smitea
 */
public interface Future<T> {
    T await() throws Exception;
    T await(long amount, TimeUnit unit, String msg) throws Exception;
    void then(Callback<T> callback);
}
