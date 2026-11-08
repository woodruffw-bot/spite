const value = await Promise.resolve(42);
if (this !== undefined || value !== 42) throw value;
export { value };
