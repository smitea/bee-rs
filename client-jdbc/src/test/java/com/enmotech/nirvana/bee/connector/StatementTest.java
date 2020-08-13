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

import static org.junit.Assert.assertArrayEquals;

public class StatementTest extends ConnectorUrl {

    private Connection createConnection() throws BeeException {
        return new BeeConnection(createClientInfo());
    }

    @Test
    public void testForIntSQL() throws SQLException {
        try (Connection connection = createConnection()) {
            Statement statement = connection.createStatement();
            statement.setQueryTimeout(10);
            ResultSet resultSet = statement.executeQuery("            SELECT  get(output,0,'TEXT','') as filesystem,\n" +
                    "                    get(output,1,'INT',0) as total,\n" +
                    "                    get(output,2,'INT',0) as used,\n" +
                    "                    get(output,3,'INT',0) as avail\n" +
                    "            FROM (SELECT split_space(line) as output FROM remote_shell('df -k',10) \n" +
                    "            WHERE line NOT LIKE '%Filesystem%' AND line NOT LIKE '%tmp%')");
            ResultSetMetaData metaData = resultSet.getMetaData();
            int colCount = metaData.getColumnCount();
            List<String> cols = new ArrayList<>();
            for (int i = 0; i < colCount; i++) {
                cols.add(metaData.getColumnLabel(i));
            }
            String[] colNames = new String[colCount];
            cols.toArray(colNames);
            assertArrayEquals(new String[]{"filesystem", "total", "used", "avail"}, colNames);
            while (resultSet.next()) {
                String filesystem = resultSet.getString("filesystem");
                long total = resultSet.getLong("total");
                long used = resultSet.getLong("used");
                long avail = resultSet.getLong("avail");

                System.out.println("filesystem:" + filesystem);
                System.out.println("total:" + total);
                System.out.println("used:" + used);
                System.out.println("avail:" + avail);

                System.out.println();
            }
        }
    }

    @Test
    public void testForDoubleSQL() throws SQLException {
        try (Connection connection = createConnection()) {
            Statement statement = connection.createStatement();
            statement.setQueryTimeout(10);
            ResultSet resultSet = statement.executeQuery("SELECT  get(output,12,'REAL',0.0) as user,\n" +
                    "                    get(output,13,'REAL',0.0) as system,\n" +
                    "                    get(output,15,'REAL',0.0) as iowait,\n" +
                    "                    get(output,14,'REAL',0.0) as idle \n" +
                    "            FROM (SELECT split_space(line) as output FROM remote_shell('vmstat 1 2',10) WHERE line_num > 2)");
            ResultSetMetaData metaData = resultSet.getMetaData();
            int colCount = metaData.getColumnCount();
            List<String> cols = new ArrayList<>();
            for (int i = 0; i < colCount; i++) {
                cols.add(metaData.getColumnLabel(i));
            }
            String[] colNames = new String[colCount];
            cols.toArray(colNames);
            assertArrayEquals(new String[]{"user", "system", "iowait", "idle"}, colNames);
            while (resultSet.next()) {
                double user = resultSet.getDouble("user");
                double system = resultSet.getDouble("system");
                double iowait = resultSet.getDouble("iowait");
                double idle = resultSet.getDouble("idle");

                System.out.println("user:" + user);
                System.out.println("system:" + system);
                System.out.println("iowait:" + iowait);
                System.out.println("idle:" + idle);

                System.out.println();
            }
        }
    }
}
