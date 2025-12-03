use std::ops::Range;

/// A square matrix split into layers.
/// This might be useful if the dimensions of the matrix need to change.
/// This way increasing/decreasing its size results in just adding/removing a layer.
/// So only the end of the vector is modified without shifting the remaining data.
///
/// row/column index:   in-layer index:
/// 0 | 0 | 0 | ...     0 | 2 | 4 | ...
/// --/   |   |         --/   |   |
/// 0   1 | 1           0   1 | 3
/// ------/             ------/
/// 0   1   2           0   1   2
///
/// previous layer size:
/// 0 | 1 | 4 | ...
/// --/   |   |
/// 1   1 | 4
/// ------/
/// 4   4   4
///
/// global index = in_layer_index + previous layer size:
/// 0 | 3 | 8 | ...
/// --/   |   |
/// 1   2 | 7
/// ------/
/// 4   5   6
pub struct LayeredMatrix<T> {
	items: Vec<T>,
	size: usize,
}
#[derive(Clone)]
struct LayerCoordinates {
	/// The index of the layer.
	layer: usize,
	/// The index of the element inside the layer counting from the first element (0) in the layer.
	index: usize,
}
impl LayerCoordinates {
	pub fn new(layer: usize, index: usize) -> Self {
		Self { layer, index }
	}
	pub fn zero() -> Self {
		Self::new(0, 0)
	}
	/// Creates new coordinates for the [linear](Self::linear) first element in the specified layer.
	pub fn first_in_layer(layer: usize) -> Self {
		Self::new(layer, 0)
	}
	/// Creates new coorinates for the element in the specified column in the specified layer.
	///
	/// Returns [Err] if the specified col is outside the bounds of the specified layer.
	pub fn from_col_in_layer(col: usize, layer: usize) -> Result<Self, ()> {
		if col > layer {
			return Err(());
		}
		Ok(Self::new(layer, col))
	}
	/// Creates new coorinates for the element in the specified row in the specified layer.
	///
	/// Returns [Err] if the specified row is outside the bounds of the specified layer.
	pub fn from_row_in_layer(row: usize, layer: usize) -> Result<Self, ()> {
		if row > layer {
			return Err(());
		}
		Ok(Self::new(layer, 2 * layer - row))
	}
	pub fn from_row_col(row: usize, col: usize) -> Self {
		let (layer, index) = match (row, col) {
			(row, col) if col <= row => (row, col),
			(row, col) => (col, 2 * col - row),
		};
		Self { layer, index }
	}
	pub fn from_linear(linear: usize) -> Self {
		let layer = linear.isqrt();
		let index = linear - layer.pow(2);
		Self { layer, index }
	}

	/// The index of the layer.
	pub fn layer(&self) -> usize {
		self.layer
	}
	/// The index of the element inside the layer counting from the first element (0) in the layer.
	pub fn index(&self) -> usize {
		self.index
	}
	/// The number of elements in the layer specified in these coordinates.
	pub fn layer_size(&self) -> usize {
		2 * self.layer + 1
	}
	/// The [linear](Self::linear) index of the first element this layer.
	pub fn linear_layer_start(&self) -> usize {
		self.layer.pow(2)
	}
	/// The [linear](Self::linear) [Range] of indices of the elements in this layer.
	pub fn linear_layer_range(&self) -> Range<usize> {
		self.linear_layer_start()..self.linear_layer_start() + self.layer_size()
	}
	/// Index in the actual underlying vector.
	pub fn linear(&self) -> usize {
		self.linear_layer_start() + self.index
	}
	/// Row and column (in that order) of the represented matrix.
	pub fn row_col(&self) -> (usize, usize) {
		let layer_size = 2 * self.layer + 1;
		let (row, col) = match self.index {
			idx if idx <= self.layer => (self.layer, idx),
			idx => (layer_size - idx - 1, self.layer),
		};
		(row, col)
	}

	pub fn linear_increment(&mut self) {
		self.index += 1;
		if self.index >= self.layer_size() {
			self.layer += 1;
			self.index = 0;
		}
	}
}
impl Default for LayerCoordinates {
	fn default() -> Self {
		Self::zero()
	}
}

impl<T: Default> LayeredMatrix<T> {
	/// Create a new [LayeredMatrix] of specified size and fill it with [T]s [default](Default::default) value.
	pub fn new(size: usize) -> Self {
		Self { items: (1..size.pow(2)).map(|_| T::default()).collect(), size }
	}
}
impl<T: Default> LayeredMatrix<T> {
	/// Adds or removes rows and columns at the end to meet the specified size.
	pub fn resize(&mut self, size: usize) {
		self.size = size;
		let linear_target_length = self.size.pow(2);
		self.items.resize_with(linear_target_length, Default::default);
	}
}
impl<T: Clone> LayeredMatrix<T> {
	/// Adds or removes rows and columns at the end to meet the specified size.
	/// New fields are filed with the provided value.
	pub fn resize_with(&mut self, size: usize, value: T) {
		self.size = size;
		let linear_target_length = self.size.pow(2);
		self.items.resize(linear_target_length, value);
	}
}
impl<T> LayeredMatrix<T> {
	/// Create a new [LayeredMatrix] of specified size and fill it using the provided generator.
	pub fn generate(size: usize, mut generator: impl FnMut(usize, usize) -> T) -> Self {
		let items = (1..size.pow(2))
			.map(|linear| {
				let (row, col) = LayerCoordinates::from_linear(linear).row_col();
				generator(row, col)
			})
			.collect();
		Self { items, size }
	}

	/// Gets the specified entry if the entry exists.
	pub fn entry(&self, row: usize, col: usize) -> Result<&T, ()> {
		let global_index = LayerCoordinates::from_row_col(row, col).linear();
		self.items.get(global_index).ok_or(())
	}
	/// Gets the specified entry if the entry exists.
	pub fn entry_mut(&mut self, row: usize, col: usize) -> Result<&mut T, ()> {
		let linear_index = LayerCoordinates::from_row_col(row, col).linear();
		self.items.get_mut(linear_index).ok_or(())
	}
	pub fn foreach(&self, mut action: impl FnMut(usize, usize, &T)) {
		let mut coordinates = LayerCoordinates::zero();
		for item in self.items.iter() {
			let (row, col) = coordinates.row_col();
			action(row, col, item);
			coordinates.linear_increment();
		}
	}
	pub fn linear_iter(&self) -> LinearLayeredMatrixIterator<T> {
		LinearLayeredMatrixIterator::new(&self)
	}
	// OPTIMIZE: this is O(n^2) and could be O(n)
	pub fn remove_row_and_column(&mut self, index: usize) -> Result<(), ()> {
		if index >= self.size {
			return Err(()); // the remove functions may panic if the range is misspecified
		}
		// order of removal is vital here: we are starting from the back
		for layer in (index + 1..self.size).rev() {
			// we should never get an error here because the loop is specified such that layer is always bigger than index
			self.remove_row_from_layer(index, layer).expect("logic error");
			self.remove_col_from_layer(index, layer).expect("logic error");
		}
		self.remove_layer(index);
		self.size -= 1;
		Ok(())
	}

	/// May panic if layer is out of bounds.
	fn remove_layer(&mut self, layer: usize) {
		let layer = LayerCoordinates::first_in_layer(layer);
		self.items.drain(layer.linear_layer_range());
	}
	/// May panic if col or layer are out of bounds.
	fn remove_col_from_layer(&mut self, col: usize, layer: usize) -> Result<(), ()> {
		self.items.remove(LayerCoordinates::from_col_in_layer(col, layer)?.linear());
		Ok(())
	}
	/// May panic if row or layer are out of bounds.
	fn remove_row_from_layer(&mut self, row: usize, layer: usize) -> Result<(), ()> {
		self.items.remove(LayerCoordinates::from_row_in_layer(row, layer)?.linear());
		Ok(())
	}
}
impl<T: Clone> Clone for LayeredMatrix<T> {
	fn clone(&self) -> Self {
		Self { items: self.items.clone(), size: self.size.clone() }
	}
}

#[derive(Clone)]
pub struct LinearLayeredMatrixIterator<'a, T> {
	matrix: &'a LayeredMatrix<T>,
	current: usize,
	coordinates: LayerCoordinates,
}
impl<'a, T> LinearLayeredMatrixIterator<'a, T> {
	fn new(matrix: &'a LayeredMatrix<T>) -> Self {
		Self { matrix, current: 0, coordinates: LayerCoordinates::zero() }
	}
}
impl<'a, T> Iterator for LinearLayeredMatrixIterator<'a, T> {
	type Item = (usize, usize, &'a T);
	fn next(&mut self) -> Option<Self::Item> {
		let item = self.matrix.items.get(self.current)?;
		let (row, col) = self.coordinates.row_col();
		self.coordinates.linear_increment();
		return Some((row, col, item));
	}
}
