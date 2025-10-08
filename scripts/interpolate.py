import numpy as np
from numpy.polynomial import chebyshev

np.set_printoptions(suppress=True,precision=100)

def find_minimal_polynomial(max_error):
    # Test different degrees
    for degree in range(2, 12):
        # Get Chebyshev coefficients for cosine approximation
        func = lambda x: np.log(x / 2 + 1.5)
        cheb = chebyshev.chebinterpolate(func, degree)
        coeffs = chebyshev.cheb2poly(cheb)

        def poly(x, coeffs):
            result = 0
            for i, c in enumerate(coeffs):
                result += c * x**i
            return result

        x_test = np.linspace(-1, 1, 1000)
        error = np.abs(poly(x_test, coeffs) - func(x_test))
        max_err = np.max(error)
        
        print(f"{coeffs}")
        # print(f"{x_test}")
        # print(f"{poly(x_test, coeffs)}")

        print(f"Degree {degree}: max error = {max_err:.6f}")

        if max_err <= max_error:
            print(f"\nFound minimal degree: {degree}")
            return poly, degree

    return None, None

# Find the solution
poly, degree = find_minimal_polynomial(0.00001)