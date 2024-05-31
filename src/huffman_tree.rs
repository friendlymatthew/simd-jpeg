use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// https://www.youtube.com/watch?v=wLoWd2KyUro
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TableType {
    AC = 1,
    DC = 0,
}

impl TableType {
    fn from(ht_type: u8) -> Self {
        match ht_type {
            1 => TableType::AC,
            0 => TableType::DC,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Eq)]
pub(crate) struct HeapItem {
    freq: usize,
    node: NPtr,
}

impl HeapItem {
    fn from(freq: usize, node: NPtr) -> Self {
        HeapItem { freq, node }
    }
}

impl From<(usize, NPtr)> for HeapItem {
    fn from(tuple: (usize, NPtr)) -> Self {
        HeapItem {
            freq: tuple.0,
            node: tuple.1,
        }
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}

struct HuffmanNode {
    pub(crate) code: u8,
    pub(crate) freq: usize,
    pub(crate) left: NPtr,
    pub(crate) right: NPtr,
}

impl HuffmanNode {
    fn new_node(code: u8, freq: usize) -> Self {
        HuffmanNode {
            code,
            freq,
            left: None,
            right: None,
        }
    }

    fn is_leaf(ptr: NPtr) -> bool {
        if let Some(node) = ptr {
            unsafe { (*node.as_ptr()).code != u8::MAX }
        } else {
            unreachable!()
        }
    }
}

type NPtr = Option<NonNull<HuffmanNode>>;

#[derive(Debug)]
pub struct HuffmanTree {
    pub(crate) h_type: TableType,
    pub(crate) h_id: usize,
    root: NPtr,
    _woof: PhantomData<HuffmanNode>,
}

impl HuffmanTree {
    pub fn from(ht_type: u8, ht_id: usize, code_freqs: Vec<(u8, usize)>) -> Self {
        let mut min_heap = BinaryHeap::new();

        for (code, freq) in code_freqs {
            let new_node = unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(HuffmanNode::new_node(code, freq))))
            };

            min_heap.push(HeapItem::from(freq, Some(new_node)))
        }

        while min_heap.len() > 1 {
            let left = min_heap.pop();

            if min_heap.len() == 1 {
                break;
            }

            let right = min_heap.pop();

            match (left, right) {
                (Some(left_item), Some(right_item)) => {
                    let sum_freq = left_item.freq + right_item.freq;
                    let new_node = unsafe {
                        NonNull::new_unchecked(Box::into_raw(Box::new(HuffmanNode {
                            freq: sum_freq,
                            code: u8::MAX,
                            right: right_item.node,
                            left: left_item.node,
                        })))
                    };

                    min_heap.push(HeapItem::from(sum_freq, Some(new_node)))
                }
                _ => unreachable!(),
            }
        }

        let root_item = min_heap.pop();
        debug_assert!(root_item.is_some());
        let HeapItem { node: root, .. } = root_item.unwrap();

        HuffmanTree {
            root,
            h_id: ht_id,
            h_type: TableType::from(ht_type),
            _woof: PhantomData,
        }
    }

    fn autumn(ptr: NPtr) {
        if let Some(node) = ptr {
            unsafe {
                let left = (*node.as_ptr()).left;
                let right = (*node.as_ptr()).right;

                HuffmanTree::autumn(left);
                HuffmanTree::autumn(right);

                let _ = Box::from_raw(node.as_ptr());
            }
        }
    }
}

impl Drop for HuffmanTree {
    fn drop(&mut self) {
        HuffmanTree::autumn(self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_tree_construction() -> Result<()> {
        let code_freqs = vec![(1, 5), (2, 9), (3, 12), (4, 13), (5, 16), (6, 45)];

        let tree = HuffmanTree::from(1, 1, code_freqs);

        assert!(tree.root.is_some());
        let tree = tree.root.unwrap();

        assert!(!HuffmanNode::is_leaf(Some(tree)));
        assert!(unsafe { (*tree.as_ptr()).left.is_some() });
        assert!(unsafe { (*tree.as_ptr()).right.is_some() });

        Ok(())
    }

    #[test]
    fn test_min_heap() -> Result<()> {
        let mut min_heap = BinaryHeap::new();

        for i in 36..0 {
            min_heap.push(HeapItem {
                freq: i,
                node: Some(unsafe {
                    NonNull::new_unchecked(Box::into_raw(Box::new(HuffmanNode::new_node(
                        i as u8, i,
                    ))))
                }),
            })
        }

        let mut expected = 36;
        while !min_heap.is_empty() {
            let res = min_heap.pop();
            assert!(res.is_some());
            let HeapItem { freq, .. } = res.unwrap();

            assert_eq!(expected, freq);

            expected -= 1;
        }

        Ok(())
    }
}
