use tokio::net::{TcpStream, ToSocketAddrs};
use crate::error::ConnectionError;
use crate::kernel::CommandData;
use crate::KvsError;
use crate::net::connection::Connection;
use crate::net::{Result, CommandOption};

pub struct Client {
    connection: Connection
}

impl Client {
    /// 与客户端进行连接
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
        let socket = TcpStream::connect(addr).await?;

        let connection = Connection::new(socket);

        Ok(Client{
            connection
        })
    }

    /// 存入数据
    pub async fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()>{
        self.send_cmd(CommandOption::Cmd(CommandData::set(key, value))).await?;
        Ok(())
    }

    /// 删除数据
    pub async fn remove(&mut self, key: Vec<u8>) -> Result<()>{
        self.send_cmd(CommandOption::Cmd(CommandData::remove(key))).await?;
        Ok(())
    }

    /// 获取数据
    pub async fn get(&mut self, key: Vec<u8>) -> Result<Option<Vec<u8>>>{
        match self.send_cmd(CommandOption::Cmd(CommandData::get(key))).await? {
            CommandOption::Value(vec) => Ok(Some(vec)),
            _ => Err(ConnectionError::KvStoreError(KvsError::NotMatchCmd))
        }
    }

    /// 批量处理
    pub async fn batch(&mut self, batch_cmd: Vec<CommandData>, is_parallel: bool) -> Result<Vec<Option<Vec<u8>>>>{
        match self.send_cmd(CommandOption::VecCmd(batch_cmd, is_parallel)).await? {
            CommandOption::ValueVec(vec) => Ok(vec),
            _ => Err(ConnectionError::KvStoreError(KvsError::NotMatchCmd))
        }
    }

    /// 磁盘占用
    pub async fn size_of_disk(&mut self) -> Result<u64> {
        match self.send_cmd(CommandOption::SizeOfDisk(0)).await? {
            CommandOption::SizeOfDisk(size_of_disk) => {Ok(size_of_disk)},
            _ => Err(ConnectionError::KvStoreError(KvsError::NotMatchCmd))
        }
    }

    async fn send_cmd(&mut self, cmd_option: CommandOption) -> Result<CommandOption>{
        self.connection.write(cmd_option).await?;
        Ok(self.connection.read().await?)
    }
}