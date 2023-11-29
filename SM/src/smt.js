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

module.exports = class SMT {

  /**
   * Constructor Sparse-merkle-tree
   * @param {Object} db - Database to use
   * @param {Object} hash - hash function
   * @param {Field} F - Field element
   */
  constructor(db, hash, F) {
    this.db = db;
    this.hash = hash;
    this.F = F;
    this.empty = [F.zero, F.zero, F.zero, F.zero];
  }
  /**
   * Insert node into the merkle-tree
   * @param {Array[Field]} oldRoot - previous root
   * @param {Array[Field]} key - path merkle-tree to insert the value
   * @param {Scalar} value - value to insert
   * @returns {Object} Information about the tree insertion
   *      {Array[Field]} oldRoot: previous root,
   *      {Array[Field]} newRoot: new root
   *      {Array[Field]} key modified,
   *      {Array[Array[Field]]} siblings: array of siblings,
   *      {Array[Field]} insKey: inserted key,
   *      {Scalar} insValue: insefted value,
   *      {Bool} isOld0: is new insert or delete,
   *      {Scalar} oldValue: old leaf value,
   *      {Scalar} newValue: new leaf value,
   *      {String} mode: action performed by the insertion,
   *      {Number} proofHashCounter: counter of hashs must be done to proof this operation
   */
  async set(oldRoot, key, value) {
    console.log("SMT set oldRoot, key, value: ", oldRoot, key, value)
    if (oldRoot === undefined) {
      console.log("oldRoot is undefined")
      oldRoot = {
        fe0: 0,
        fe1: 0,
        fe2: 0,
        fe3: 0,
      }
    }

    let setRequest = {
      old_root: oldRoot,
      key: key,
      value: value
    }

    let setResponse = await new Promise((resolve, reject) => {
      client.Set(setRequest, function (err, response) {
        if (err) {
          console.log("err: ", err)
          reject(err)
        } else {
          resolve(response)
        }
      });
    })
    return setResponse
  }


  /**
   * Get value merkle-tree
   * @param {Array[Field]} root - merkle-tree root
   * @param {Array[Field]} key - path to retoreve the value
   * @returns {Object} Information about the value to retrieve
   *      {Array[Field]} root: merkle-tree root,
   *      {Array[Field]} key: key to look for,
   *      {Scalar} value: value retrieved,
   *      {Array[Array[Field]]} siblings: array of siblings,
   *      {Bool} isOld0: is new insert or delete,
   *      {Array[Field]} insKey: key found,
   *      {Scalar} insValue: value found,
   *      {Number} proofHashCounter: counter of hashs must be done to proof this operation
   */
  async get(root, key) {
    console.log("SMT get key: ", key)
    let getRequest = {
      root: root,
      key: key,
    }

    let getResponse = await new Promise((resolve, reject) => {
      client.Get(getRequest, function (err, response) {
        if (err) {
          console.log("err: ", err)
          reject(err);
        } else {
          resolve(response);
        }
      })
    })
    return getResponse
  }
}