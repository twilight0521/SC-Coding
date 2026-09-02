export interface DiffSegment {
  type: 'equal' | 'insert' | 'delete';
  lines: string[];
}

export function computeDiff(original: string, modified: string): DiffSegment[] {
  const originalLines = original.split('\n');
  const modifiedLines = modified.split('\n');
  const result: DiffSegment[] = [];
  let i = 0;
  let j = 0;

  while (i < originalLines.length || j < modifiedLines.length) {
    if (i < originalLines.length && j < modifiedLines.length) {
      if (originalLines[i] === modifiedLines[j]) {
        result.push({ type: 'equal', lines: [originalLines[i]] });
        i++;
        j++;
      } else {
        let nextMatchI = -1;
        let nextMatchJ = -1;

        for (let k = i; k < originalLines.length; k++) {
          for (let l = j; l < modifiedLines.length; l++) {
            if (originalLines[k] === modifiedLines[l]) {
              nextMatchI = k;
              nextMatchJ = l;
              break;
            }
          }
          if (nextMatchI !== -1) break;
        }

        if (nextMatchI === -1) {
          const deleted = originalLines.slice(i);
          const inserted = modifiedLines.slice(j);
          if (deleted.length > 0) result.push({ type: 'delete', lines: deleted });
          if (inserted.length > 0) result.push({ type: 'insert', lines: inserted });
          i = originalLines.length;
          j = modifiedLines.length;
        } else {
          if (nextMatchI > i) {
            result.push({ type: 'delete', lines: originalLines.slice(i, nextMatchI) });
            i = nextMatchI;
          }
          if (nextMatchJ > j) {
            result.push({ type: 'insert', lines: modifiedLines.slice(j, nextMatchJ) });
            j = nextMatchJ;
          }
        }
      }
    } else if (i < originalLines.length) {
      result.push({ type: 'delete', lines: originalLines.slice(i) });
      i = originalLines.length;
    } else {
      result.push({ type: 'insert', lines: modifiedLines.slice(j) });
      j = modifiedLines.length;
    }
  }

  return result;
}
