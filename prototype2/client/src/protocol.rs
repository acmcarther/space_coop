#[derive(PartialEq, Debug, Clone)]
pub enum InternalClientEvent {
  Exit,
  CameraMove(CameraDir),
  CameraRot(CameraOrient),
}

#[derive(PartialEq, Debug, Clone)]
pub enum CameraDir {
  Forward,
  Backward,
  Left,
  Right,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CameraOrient {
  PitchUp,
  PitchDown,
  YawLeft,
  YawRight,
}
