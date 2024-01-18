use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use rand::{seq::IteratorRandom, thread_rng};
use strength_reduce::StrengthReducedU32;

pub struct PatchMaskGenerator {
    ratio: f32,
    patch_size: u32,
}

impl PatchMaskGenerator {
    #[inline]
    pub fn new(ratio: f32, patch_size: u32) -> Self {
        Self { ratio, patch_size }
    }

    #[inline]
    pub fn transform(&self, mut image: DynamicImage) -> DynamicImage {
        let (width, height) = image.dimensions();

        let patch_size = self.patch_size;
        // Calculate the number of patches in each dimension
        let patches_per_row = width / patch_size;
        let patches_per_col = height / patch_size;

        // Calculate the total number of patches
        let num_patches = patches_per_row * patches_per_col;

        // Calculate the number of patches to mask using rayon
        let mask_patches = ((num_patches as f32 * self.ratio) as usize).min(num_patches as usize);

        let mut rng = thread_rng();

        let selected_indices: Vec<u32> = (0..num_patches).choose_multiple(&mut rng, mask_patches);

        let reduced_divisor_patch_per_row = StrengthReducedU32::new(patches_per_row);
        let reduced_modulo_patch_per_row = StrengthReducedU32::new(patches_per_row);

        for index in selected_indices {
            let patch_row = index / reduced_divisor_patch_per_row;
            let patch_col = index % reduced_modulo_patch_per_row;

            // Calculate the starting coordinates of the patch
            let start_x = patch_col * patch_size;
            let start_y = patch_row * patch_size;

            // Process each pixel in the patch
            for y in start_y..start_y + patch_size {
                for x in start_x..start_x + patch_size {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }

        image
    }
}
