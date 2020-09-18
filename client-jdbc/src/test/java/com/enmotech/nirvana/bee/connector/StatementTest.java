package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.ConnectionFactory;
import org.junit.Test;

import java.sql.Connection;
import java.sql.ResultSet;
import java.sql.ResultSetMetaData;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.ArrayList;
import java.util.List;

import static org.junit.Assert.assertArrayEquals;

public class StatementTest extends ConnectionFactory {

    private Connection getRemoteConnection(BeeDatasource.SessionMode mode) throws SQLException {
        return createRemoteDatasource(mode).getConnection();
    }

    private Connection getAgentConnection(BeeDatasource.SessionMode mode) throws SQLException {
        return createClientAgentInfo(mode).getConnection();
    }

    @Test
    public void testForIntSQL() throws SQLException {
        try (Connection connection = getAgentConnection(BeeDatasource.SessionMode.SQLITE)) {
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

    @Test
    public void testForIntLua() throws SQLException {
        try (Connection connection = getAgentConnection(BeeDatasource.SessionMode.LUA)) {
            Statement statement = connection.createStatement();
            statement.setQueryTimeout(10);
            ResultSet resultSet = statement.executeQuery("local resp=filesystem();\n" +
                    "            while(resp:has_next())\n" +
                    "            do\n" +
                    "                _request:commit(_next);\n" +
                    "            end");
            ResultSetMetaData metaData = resultSet.getMetaData();
            int colCount = metaData.getColumnCount();
            List<String> cols = new ArrayList<>();
            for (int i = 0; i < colCount; i++) {
                cols.add(metaData.getColumnLabel(i));
            }
            String[] colNames = new String[colCount];
            cols.toArray(colNames);
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
