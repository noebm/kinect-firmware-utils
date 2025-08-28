{
  crane-lib,
}:

crane-lib.buildPackage {
  src = crane-lib.cleanCargoSource (crane-lib.path ../.);
}
