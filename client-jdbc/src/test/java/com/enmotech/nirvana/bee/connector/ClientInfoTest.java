package com.enmotech.nirvana.bee.connector;

import org.junit.Test;

import java.sql.SQLException;
import java.util.Properties;

import static org.junit.Assert.assertEquals;

public class ClientInfoTest {

    @Test
    public void test() throws SQLException {
        ClientInfo clientInfo = new ClientInfo("sqlite:remote:password://127.0.0.1:22/bee?connection_timeout=10&application=jdbc",new Properties());
        assertEquals("jdbc", clientInfo.getApplication());
    }

    @Test
    public void testUrl1() throws SQLException{
        new ClientInfo("sqlite:remote:password://oracle:admin@127.0.0.1:20002/bee?connect_timeout=5",new Properties());
    }

    @Test
    public void testUrl2() throws SQLException{
        new ClientInfo("sqlite:remote:password://oracle:admin@127.0.0.1:20002/bee",new Properties());
    }

    @Test
    public void testUrl3() throws SQLException{
        new ClientInfo("sqlite:remote:password://oracle:admin@127.0.0.1:20002",new Properties());
    }

    @Test
    public void testUrl4() throws SQLException{
        new ClientInfo("sqlite:remote:password://oracle@127.0.0.1:20002",new Properties());
    }

    @Test
    public void testUrl5() throws SQLException{
        new ClientInfo("sqlite:remote:password://127.0.0.1:20002",new Properties());
    }

    @Test
    public void testUrl6() throws SQLException{
        new ClientInfo("sqlite:remote:pubkey://127.0.0.1:20002",new Properties());
    }

    @Test
    public void testUrl7() throws SQLException{
        new ClientInfo("sqlite:remote:pubkey://127.0.0.1:20002",new Properties());
    }

    @Test
    public void testUrl8() throws SQLException{
        ClientInfo clientInfo = new ClientInfo("sqlite:remote:pubkey://127.0.0.1:20002?username=oracle&password=admin",new Properties());
        assertEquals("oracle",clientInfo.getUsername());
    }
}
