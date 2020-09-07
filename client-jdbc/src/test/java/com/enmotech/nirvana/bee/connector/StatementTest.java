package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import org.junit.Test;

import java.sql.Connection;
import java.sql.ResultSet;
import java.sql.ResultSetMetaData;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.ArrayList;
import java.util.List;
import java.util.Queue;
import java.util.concurrent.BlockingDeque;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.LinkedBlockingDeque;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;

import static org.junit.Assert.assertArrayEquals;

public class StatementTest extends ConnectorUrl {

    private Connection createConnection() throws BeeException {
        return new BeeConnection(createClientAgentInfo());
    }

    @Test
    public void testForIntSQL() throws SQLException {
        try (Connection connection = createConnection()) {
            Statement statement = connection.createStatement();
            statement.setQueryTimeout(10);
            ResultSet resultSet = statement.executeQuery("SELECT *FROM filesystem");
            ResultSetMetaData metaData = resultSet.getMetaData();
            int colCount = metaData.getColumnCount();
            List<String> cols = new ArrayList<>();
            for (int i = 0; i < colCount; i++) {
                cols.add(metaData.getColumnLabel(i));
            }
            String[] colNames = new String[colCount];
            cols.toArray(colNames);
            assertArrayEquals(new String[]{"name", "mount_on", "total_bytes", "used_bytes", "free_bytes"}, colNames);
            while (resultSet.next()) {
                String filesystem = resultSet.getString("name");
                long total = resultSet.getLong("total_bytes");
                long used = resultSet.getLong("used_bytes");
                long avail = resultSet.getLong("free_bytes");

                System.out.println("name:" + filesystem);
                System.out.println("total:" + total);
                System.out.println("used:" + used);
                System.out.println("avail:" + avail);

                System.out.println();
            }
        }
    }
}
