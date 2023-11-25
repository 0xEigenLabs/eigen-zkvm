const path = require("path");
const { getPoseidon } = require('@0xpolygonhermez/zkevm-commonjs');
const dotenv = require('dotenv');
const env = dotenv.config({
  path: path.join(__dirname, '.env')
});

let PROTO_PATH = path.join(__dirname, "/../../../eigen-prover/service/proto/src/proto/statedb/v1/statedb.proto")

let grpc = require("@grpc/grpc-js");
let protoLoader = require("@grpc/proto-loader");
let packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
});

let statedb_proto = grpc.loadPackageDefinition(packageDefinition).statedb.v1;
const dbAddr = process.env.dbAddr

const client = new statedb_proto.StateDBService(
  dbAddr,
  grpc.credentials.createInsecure()
);

module.exports = class StateDB {

  /**
   * Constructor StateDB
   * @param {Field} F - Field element
   */
  constructor(F) {
    this.F = F;
  }

  /**
   * Get program value
   * @param {Array[Field]} key - key in Array Field representation
   * @returns {Array[Byte] | null} Node childs if found, otherwise return null
   */
  async getProgram(key) {
    console.log("StateDB get key: ", key)
    let getProgramRequest = {
      key: key
    }

    let value = await new Promise((resolve, reject) => {
      client.GetProgram(getProgramRequest, function (err, response) {
        if (err) {
          console.log("err: ", err)
          reject(err);
        } else {
          resolve(response.data.toString('utf-8'));
        }
      })
    })
    return value
  }

  /**
   * Set program node
   * @param {Array[Field]} key - key in Field representation
   * @param {Array[byte]} value - child array
   */
  async setProgram(key, value) {
    console.log(`StateDB set key: ${key}, value: ${value}`)
    let setProgramRequest = {
      key: key,
      data: Buffer.from(value)
    }

    await new Promise((resolve, reject) => {
      client.SetProgram(setProgramRequest, function (err, response) {
        if (err) {
          console.log("err: ", err)
          reject(err)
        } else {
          resolve(response)
        }
      })
    })
  }
}