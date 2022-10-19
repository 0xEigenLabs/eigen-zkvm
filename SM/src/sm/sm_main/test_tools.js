/* eslint-disable prefer-destructuring */

const { Scalar } = require('ffjavascript');

const { scalar2fea, fea2scalar } = require('@0xpolygonhermez/zkevm-commonjs/src/smt-utils');

class TestTools {

    log(log) {
        console.log('\x1b[36mTEST '+log+'\x1b[0m');
    }

    setup(data, evalCommand) {
        this.data = data;
        this.results = [];
        this.index = 0;
        this.evalCommand = evalCommand;
    }

    load(ctx, tag) {
        this.index = 0;
        this.current = tag.params[0].varName;
        if (!this.data[this.current]) {
            this.log('NOT FOUND test '+ this.current);
        }
        else {
            this.log('\n==== TEST '+ this.current+' ====');
            this.results[this.current] = {};
        }
        return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
    }

    input(ctx, tag, inputType, log) {
        inputType = inputType | 'INPUT';

        if (!this.data[this.current]) {
            this.log(`NOT FOUND test ${this.current} ${inputType} was zero`);
            return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }

        let name = tag.params[0].varName || tag.params[0].regName;
        let value = this.data[this.current][this.index][name];
        if (log) {
            this.log(`${inputType} (${name}): ${value}`);
        }

        if (typeof(value) === 'number' || typeof(value) === 'boolean') {
            return [ctx.Fr.e(value), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
        if (typeof(value) == 'string') {
            value = BigInt(value);
        }
        if (typeof(value) == 'bigint') {
            return scalar2fea(ctx.Fr, value);
        }
    }

    // posar al final.

    assertEquals(ctx, tag) {
        let current = this.current;
        let index = this.index;
        if (!this.data[current]) {
            this.log('NOT FOUND test '+ current +", assert hasn't been done");
            return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
        let name = tag.params[0].varName || tag.params[0].regName;
        let rightName = tag.params[1].varName || tag.params[1].regName;
        let left = BigInt(this.data[current][index][name]);
        let right = BigInt(this.evalCommand(ctx, tag.params[1]));
        const inHex = this.data[current][index][name].substr(0, 2) == '0x';

        if (!this.results[current]['#'+index]) {
            this.results[current]['#'+index] = { ok: 0, fail: 0 };
        }
        if (inHex) {
            left = left.toString(16);
            right = right.toString(16);
            if (left.length > right.length) {
                right = right.padStart(left.length, '0');
            } else {
                left = left.padStart(right.length, '0');
            }
            left = '0x'+left;
            right = '0x'+right;

        }
        if (left == right) {
            this.log(`assertEquals(${name}:${left}, ${rightName}:${right}) - \x1b[1;32m[PASS]`);
            ++this.results[current]['#'+index].ok;
        }
        else {
            this.log(`assertEquals(${name}:${left}, ${rightName}:${right}) - \x1b[1;31m[FAIL]`);
            ++this.results[current]['#'+index].fail;
        }
        return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
    }

    assertCond(ctx, tag) {
        let current = this.current;
        let index = this.index;
        if (!this.data[current]) {
            this.log('NOT FOUND test '+ current +", assert hasn't been done");
            return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
        let name = tag.params[0].varName || tag.params[0].regName;
        let left = this.data[current][index][name];
        let right = BigInt(this.evalCommand(ctx, tag.params[1]));

        if (!this.results[current]['#'+index]) {
            this.results[current]['#'+index] = { ok: 0, fail: 0 };
        }
        if (left == right) {
            this.log('assertEquals('+left+','+right+') - \x1b[1;32m[PASS]');
            ++this.results[current]['#'+index].ok;
        }
        else {
            this.log('assertEquals('+left+','+right+') - \x1b[1;31m[FAIL]');
            ++this.results[current]['#'+index].fail;
        }
        return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
    }

    next(ctx, tag) {
        if (!this.data[this.current]) {
            this.log('NOT FOUND test '+ this.current+", no continue");
            return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
        if ((this.index + 1) < this.data[this.current].length) {
            ++this.index;
            this.log('continue('+this.index+'/'+this.data[this.current].length+')');
            return [ctx.Fr.e(-1), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
        else {
            this.log('END');
            return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
    }

    summary(ctx, tag) {
        let totals = {};
        let ok = 0;
        let fail = 0;
        Object.entries(this.results).forEach(
            ([key, value]) => {
                totals[key] = { ok: 0, fail: 0 };
                Object.entries(value).forEach(
                    ([index, stats])  => {
                        if (stats.fail === 0) {
                            ++totals[key].ok;
                        }
                        else {
                            ++totals[key].fail;
                        }
                    });
                if (totals[key].fail === 0) ++ok;
                else ++fail;
            });
        let pass = (fail + ok) > 0 ? ok / (fail+ok) : 0;

        const percent = new Intl.NumberFormat('en-US',
                    { style: 'percent', maximumFractionDigits: 2,
                                         minimumFractionDigits: 2 });

        console.log(("============ TEST SUMMARY PASS "+percent.format(pass)+" ").padEnd(62, "="));
        console.log("Test                          |Result|OK     |Fail   |% Pass |");
        console.log("------------------------------|------|-------|-------|-------|");
        let totOk = 0;
        let totFail = 0;
        Object.entries(totals).forEach(
            ([key, value]) => {
                let total = value.ok + value.fail;
                totOk += value.ok;
                totFail += value.fail;
                console.log(key.padEnd(30) + '|'
                            + (value.fail == 0 ? "\x1b[32m PASS \x1b[0m": "\x1b[31m FAIL \x1b[0m") + '|'
                            + (value.ok + '/' + total).padStart(7) + '|'
                            + (value.fail + '/' + total).padStart(7) + '|'
                            + (percent.format(value.ok/total)).padStart(7) + '|');
            });
        console.log("==============================|======|=======|=======|=======|");
        console.log("TOTALS                        |"
                    + (fail == 0 ? "\x1b[32m PASS \x1b[0m": "\x1b[31m FAIL \x1b[0m") + '|'
                    + (totOk + '/' + (totOk + totFail)).padStart(7) + '|'
                    + (totFail + '/' + (totOk + totFail)).padStart(7) + '|'
                    + (percent.format(totOk/(totOk + totFail))).padStart(7) + '|');
        return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
    }
}

module.exports = new TestTools();