pub trait MotorDriver {
    type Error;

    fn drive(&mut self, command: crate::DriverCommand) -> Result<(), Self::Error>;

    fn coast(&mut self) -> Result<(), Self::Error> {
        self.drive(crate::DriverCommand::Coast)
    }

    fn brake(&mut self) -> Result<(), Self::Error> {
        self.drive(crate::DriverCommand::Brake)
    }
}
